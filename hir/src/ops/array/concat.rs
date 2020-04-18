use crate::infer::*;
use crate::internal::*;

pub use tract_core::ops::array::{ConcatSlice, TypedConcat};

/// Concat: high level concat op
#[derive(Debug, Clone, new)]
pub struct Concat {
    axis: i64,
}

impl Concat {
    fn resolve_axis(&self, rank: i64) -> TractResult<usize> {
        if 0 <= self.axis && self.axis <= rank - 1 {
            Ok(self.axis as usize)
        } else if -rank <= self.axis && self.axis < 0 {
            Ok((self.axis + rank) as usize)
        } else {
            bail!("Illegal combination of values for rank and axis: {} and {}", rank, self.axis)
        }
    }
}

impl Op for Concat {
    fn name(&self) -> Cow<str> {
        "InferenceConcat".into()
    }

    not_a_typed_op!();
    not_a_pulsed_op!();
}

impl StatelessOp for Concat {
    /// Evaluates the operation given the input tensors.
    fn eval(&self, inputs: TVec<Arc<Tensor>>) -> TractResult<TVec<Arc<Tensor>>> {
        let super_type: DatumType =
            DatumType::super_type_for(inputs.iter().map(|x| x.datum_type()))
                .ok_or_else(|| format!("No supertype found for {:?}", inputs))?;
        let axis = self.resolve_axis(inputs[0].shape().len() as i64)?;
        let tensors =
            inputs.iter().map(|t| t.cast_to_dt(super_type)).collect::<TractResult<TVec<_>>>()?;
        Ok(tvec!(Tensor::stack_tensors(axis, &*tensors)?.into_arc_tensor()))
    }
}

impl InferenceRulesOp for Concat {
    fn rules<'r, 'p: 'r, 's: 'r>(
        &'s self,
        s: &mut Solver<'r>,
        inputs: &'p [TensorProxy],
        outputs: &'p [TensorProxy],
    ) -> InferenceResult {
        check_output_arity(&outputs, 1)?;
        s.equals(&outputs[0].rank, &inputs[0].rank)?;
        let n = inputs.len() as usize;
        s.equals_all((0..n).map(|i| (&inputs[i].rank).bex()).collect())?;
        s.given_all((0..n).map(|i| (&inputs[i].datum_type).bex()), move |s, dts| {
            let super_type: DatumType = DatumType::super_type_for(&dts)
                .ok_or_else(|| format!("No supertype found for {:?}", dts))?;
            s.equals(&outputs[0].datum_type, super_type)
        })?;
        s.given(&inputs[0].rank, move |s, rank| {
            let axis = self.resolve_axis(rank as i64)?;
            s.equals(
                rules::expr::SumExp::new((0..n).map(|i| (&inputs[i].shape[axis]).bex()).collect()),
                &outputs[0].shape[axis],
            )?;
            for axis in 0..axis {
                s.equals(&outputs[0].shape[axis], &inputs[0].shape[axis])?;
                s.equals_all((0..n).map(|i| inputs[i].shape[axis].bex()).collect())?;
            }
            for axis in (axis + 1)..(rank as usize) {
                s.equals(&outputs[0].shape[axis], &inputs[0].shape[axis])?;
                s.equals_all((0..n).map(|i| inputs[i].shape[axis].bex()).collect())?;
            }
            Ok(())
        })?;
        Ok(())
    }

    fn to_typed(
        &self,
        _source: &InferenceModel,
        node: &InferenceNode,
        target: &mut TypedModel,
        mapping: &HashMap<OutletId, OutletId>,
    ) -> TractResult<TVec<OutletId>> {
        let mapped_inputs =
            node.inputs.iter().map(|i| mapping[i].clone()).collect::<TVec<OutletId>>();
        let facts = mapped_inputs
            .iter()
            .map(|i| target.outlet_fact(*i).map(|x| x.clone()))
            .collect::<TractResult<TVec<_>>>()?;

        let super_type = if let Some(super_type) =
            DatumType::super_type_for(facts.iter().map(|x| x.datum_type))
        {
            super_type
        } else {
            bail!("Can not type op");
        };

        let axis = self.resolve_axis(facts[0].shape.rank() as i64)?;

        let mut slices: TVec<ConcatSlice> = tvec![];
        let mut kept_inputs: TVec<OutletId> = tvec![];
        for (ix, (fact, outlet)) in facts.iter().zip(mapped_inputs.iter()).enumerate() {
            match &fact.konst {
                Some(c_input) => {
                    slices.push(ConcatSlice::Const(
                        c_input.cast_to_dt(super_type)?.into_owned().into_arc_tensor(),
                    ));
                }
                None => {
                    let casted = target.wire_node(
                        format!("{}-Cast-{}", node.name, ix),
                        crate::ops::cast(super_type),
                        &[*outlet],
                    )?[0];
                    kept_inputs.push(casted);
                    slices.push(ConcatSlice::Var)
                }
            }
        }
        let op = TypedConcat::new(axis, slices);
        target.wire_node(&*node.name, op, &*kept_inputs)
    }

    as_op!();
}
