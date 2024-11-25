use burn_tensor::{
    ops::{TransactionOps, TransactionPrimitiveResult},
    DType, TensorData,
};

use crate::{FloatElement, IntElement, JitBackend, JitRuntime};

impl<R, F, I> TransactionOps<Self> for JitBackend<R, F, I>
where
    R: JitRuntime,
    F: FloatElement,
    I: IntElement,
{
    fn tr_execute(
        transaction: burn_tensor::ops::TransactionPrimitive<Self>,
    ) -> impl std::future::Future<Output = burn_tensor::ops::TransactionPrimitiveResult> + 'static + Send
    {
        let mut bindings = Vec::new();
        let mut client = None;

        enum Kind {
            Float(usize, Vec<usize>, DType),
            Int(usize, Vec<usize>, DType),
            Bool(usize, Vec<usize>, DType),
        }

        let mut num_bindings = 0;

        let mut kinds = Vec::new();

        transaction.read_floats.into_iter().for_each(|t| {
            if client.is_none() {
                client = Some(t.client.clone());
            }

            kinds.push(Kind::Float(num_bindings, t.shape.into(), F::dtype()));
            num_bindings += 1;
            bindings.push(t.handle.binding())
        });
        transaction.read_ints.into_iter().for_each(|t| {
            if client.is_none() {
                client = Some(t.client.clone());
            }

            kinds.push(Kind::Int(num_bindings, t.shape.into(), I::dtype()));
            num_bindings += 1;
            bindings.push(t.handle.binding())
        });
        transaction.read_bools.into_iter().for_each(|t| {
            if client.is_none() {
                client = Some(t.client.clone());
            }

            kinds.push(Kind::Bool(num_bindings, t.shape.into(), DType::U32));
            num_bindings += 1;
            bindings.push(t.handle.binding())
        });

        let client = client.unwrap();

        async move {
            let mut data = client
                .read_async(bindings)
                .await
                .into_iter()
                .map(Some)
                .collect::<Vec<_>>();

            let mut result = TransactionPrimitiveResult::default();

            for kind in kinds {
                match kind {
                    Kind::Float(index, shape, dtype) => {
                        let bytes = data.get_mut(index).unwrap().take().unwrap();
                        result.read_floats.push(TensorData {
                            bytes,
                            shape,
                            dtype,
                        });
                    }
                    Kind::Int(index, shape, dtype) => {
                        let bytes = data.get_mut(index).unwrap().take().unwrap();
                        result.read_ints.push(TensorData {
                            bytes,
                            shape,
                            dtype,
                        });
                    }
                    Kind::Bool(index, shape, dtype) => {
                        let bytes = data.get_mut(index).unwrap().take().unwrap();
                        result.read_bools.push(TensorData {
                            bytes,
                            shape,
                            dtype,
                        });
                    }
                }
            }

            result
        }
    }
}