// Language
use alloc::vec;
use alloc::vec::Vec;
use burn_tensor::ops::{BoolTensorOps, IntTensorOps};
use core::ops::Range;

// Current crate
use crate::element::FloatNdArrayElement;
use crate::NdArrayDevice;
use crate::{tensor::NdArrayTensor, NdArrayBackend};

// Workspace crates
use burn_tensor::{backend::Backend, Data, Shape};

use super::NdArrayOps;

impl<E: FloatNdArrayElement> BoolTensorOps<NdArrayBackend<E>> for NdArrayBackend<E> {
    fn bool_from_data<const D: usize>(
        data: Data<bool, D>,
        _device: &NdArrayDevice,
    ) -> NdArrayTensor<bool, D> {
        NdArrayTensor::from_data(data)
    }

    fn bool_shape<const D: usize>(
        tensor: &<NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D>,
    ) -> Shape<D> {
        tensor.shape()
    }

    fn bool_to_data<const D: usize>(
        tensor: &<NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D>,
    ) -> Data<bool, D> {
        let values = tensor.array.iter().map(Clone::clone).collect();
        Data::new(values, tensor.shape())
    }

    fn bool_into_data<const D: usize>(
        tensor: <NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D>,
    ) -> Data<bool, D> {
        let shape = tensor.shape();
        let values = tensor.array.into_iter().collect();
        Data::new(values, shape)
    }

    fn bool_to_device<const D: usize>(
        tensor: NdArrayTensor<bool, D>,
        _device: &NdArrayDevice,
    ) -> NdArrayTensor<bool, D> {
        tensor
    }

    fn bool_reshape<const D1: usize, const D2: usize>(
        tensor: NdArrayTensor<bool, D1>,
        shape: Shape<D2>,
    ) -> NdArrayTensor<bool, D2> {
        NdArrayOps::reshape(tensor, shape)
    }

    fn bool_index<const D1: usize, const D2: usize>(
        tensor: NdArrayTensor<bool, D1>,
        indexes: [Range<usize>; D2],
    ) -> NdArrayTensor<bool, D1> {
        NdArrayOps::index(tensor, indexes)
    }

    fn bool_into_int<const D: usize>(
        tensor: <NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D>,
    ) -> NdArrayTensor<i64, D> {
        let data = Self::bool_into_data(tensor);
        NdArrayBackend::<E>::int_from_data(data.convert(), &NdArrayDevice::Cpu)
    }

    fn bool_device<const D: usize>(
        _tensor: &<NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D>,
    ) -> <NdArrayBackend<E> as Backend>::Device {
        NdArrayDevice::Cpu
    }

    fn bool_empty<const D: usize>(
        shape: Shape<D>,
        _device: &<NdArrayBackend<E> as Backend>::Device,
    ) -> <NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D> {
        let values = vec![false; shape.num_elements()];
        NdArrayTensor::from_data(Data::new(values, shape))
    }

    fn bool_index_assign<const D1: usize, const D2: usize>(
        tensor: <NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D1>,
        indexes: [Range<usize>; D2],
        value: <NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D1>,
    ) -> <NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D1> {
        NdArrayOps::index_assign(tensor, indexes, value)
    }

    fn bool_cat<const D: usize>(
        tensors: Vec<<NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D>>,
        dim: usize,
    ) -> <NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D> {
        NdArrayOps::cat(tensors, dim)
    }

    fn bool_equal<const D: usize>(
        lhs: <NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D>,
        rhs: <NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D>,
    ) -> <NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D> {
        let mut array = lhs.array;
        array.zip_mut_with(&rhs.array, |a, b| *a = *a && *b);

        NdArrayTensor { array }
    }

    fn bool_equal_elem<const D: usize>(
        lhs: <NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D>,
        rhs: bool,
    ) -> <NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D> {
        let array = lhs.array.mapv(|a| a == rhs).into_shared();
        NdArrayTensor { array }
    }
    fn bool_permute<const D: usize>(
        tensor: <NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D>,
        dims: [usize; D],
    ) -> <NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D> {
        let array = tensor.array.permuted_axes(dims.to_vec());
        NdArrayTensor::new(array)
    }
    fn bool_flip<const D: usize>(
        tensor: <NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D>,
        dims: Vec<usize>,
    ) -> <NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D> {
        unimplemented!()
    }
    fn bool_upsample_bilinear2d<const D: usize, const D2: usize>(
        tensor: <NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D>,
        output_size: Vec<usize>,
        align_corners: bool,
        scales_h: impl Into<Option<f64>>,
        scales_w: impl Into<Option<f64>>,
    ) -> <NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D2> {
        unimplemented!()
    }
    fn bool_select<const D: usize, const D2: usize>(
            tensor: <NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D>,
            dim: i64,
            index: i64,
        ) -> <NdArrayBackend<E> as Backend>::BoolTensorPrimitive<D2> {
        unimplemented!()
    }
}
