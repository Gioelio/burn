use std::collections::BTreeMap;

use crate::{
    on_write::ir::{Arg, ElemwiseOp, ElemwisePrecision},
    CubeFusionHandle,
};
use burn_ir::{TensorId, TensorIr};
use cubecl::Runtime;

/// The plan is responsible to keep runtime information related to the launch of a fused kernel
/// at one place.
#[derive(Debug)]
pub(crate) struct LaunchPlan<'a, R: Runtime> {
    pub potential_inplaces: Vec<PotentialInplace<'a>>,
    pub global_inputs: Vec<TensorIr>,
    pub global_outputs: Vec<TensorIr>,
    pub handle_inputs: Vec<HandleInput<R>>,
    pub handle_outputs: Vec<HandleOutput<R>>,
    pub reference: Option<Reference>,
    pub reads: BTreeMap<TensorId, Vec<ElemwiseOp>>,
    pub writes: BTreeMap<TensorId, ElemwiseOp>,
    pub vectorization: BTreeMap<TensorId, u8>,
    pub rank: usize,
}

impl<R: Runtime> LaunchPlan<'_, R> {
    pub fn new(
        reads: &BTreeMap<TensorId, Vec<ElemwiseOp>>,
        writes: &BTreeMap<TensorId, ElemwiseOp>,
        rank: usize,
    ) -> Self {
        LaunchPlan {
            potential_inplaces: Vec::new(),
            global_inputs: Vec::new(),
            global_outputs: Vec::new(),
            handle_inputs: Vec::new(),
            handle_outputs: Vec::new(),
            reference: None,
            vectorization: BTreeMap::default(),
            reads: reads.clone(),
            writes: writes.clone(),
            rank,
        }
    }
}

#[derive(Debug)]
pub enum HandleOutput<R: Runtime> {
    Alias {
        input_pos: usize,
        precision: ElemwisePrecision,
    },
    Owned {
        global_id: TensorId,
        precision: ElemwisePrecision,
        handle: CubeFusionHandle<R>,
        global_shape: Vec<usize>,
        vectorization: u8,
    },
}

#[derive(Debug)]
pub struct HandleInput<R: Runtime> {
    pub relative_id: TensorId,
    pub global_id: TensorId,
    pub precision: ElemwisePrecision,
    pub handle: CubeFusionHandle<R>,
    pub global_shape: Vec<usize>,
    pub vectorization: u8,
}

#[derive(Debug)]
pub struct Reference {
    pub layout: Arg,
    pub shape: Vec<usize>,
    pub strides: Vec<usize>,
}

#[derive(Debug)]
pub struct PotentialInplace<'a> {
    pub input_pos: usize,
    pub tensor_relative: &'a TensorIr,
    pub strides: Vec<usize>,
}
