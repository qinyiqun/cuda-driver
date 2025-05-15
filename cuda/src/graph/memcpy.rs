﻿use super::{Graph, GraphNode, MemcpyNode, collect_dependencies};
use crate::{
    DevByte,
    // bindings::{hc_Memcpy3D, hcMemcpy3DParms, hcMemoryType},
    bindings::hcMemcpy3DParms,
};
use context_spore::AsRaw;
use std::{marker::PhantomData, mem::MaybeUninit, ptr::null_mut};

// const CFG: hc_Memcpy3D = hc_Memcpy3D {
//     srcXInBytes: 0,
//     srcY: 0,
//     srcZ: 0,
//     srcLOD: 0,
//     srcMemoryType: hcMemoryType::hcMemoryTypeDevice,
//     srcHost: null(),
//     srcDevice: null_mut(),
//     srcArray: null_mut(),
//     reserved0: null_mut(),
//     srcPitch: 0,
//     srcHeight: 0,
//     dstXInBytes: 0,
//     dstY: 0,
//     dstZ: 0,
//     dstLOD: 0,
//     dstMemoryType: hcMemoryType::hcMemoryTypeDevice,
//     dstHost: null_mut(),
//     dstDevice: null_mut(),
//     dstArray: null_mut(),
//     reserved1: null_mut(),
//     dstPitch: 0,
//     dstHeight: 0,
//     WidthInBytes: 0,
//     Height: 1,
//     Depth: 1,
// };

impl Graph {
    pub fn add_memcpy_d2d<'a>(
        &self,
        dst: &mut [DevByte],
        src: &[DevByte],
        _deps: impl IntoIterator<Item = &'a GraphNode<'a>>,
    ) -> MemcpyNode {
        assert_eq!(size_of_val(dst), size_of_val(src));
        todo!()
    }

    pub fn add_memcpy_node<'a>(
        &self,
        node: &MemcpyNode,
        deps: impl IntoIterator<Item = &'a GraphNode<'a>>,
    ) -> MemcpyNode {
        let mut params = MaybeUninit::uninit();
        driver!(hcGraphMemcpyNodeGetParams(
            node.as_raw(),
            params.as_mut_ptr()
        ));

        self.add_memcpy_node_with_params(unsafe { params.assume_init_ref() }, deps)
    }

    pub fn add_memcpy_node_with_params<'a>(
        &self,
        params: &hcMemcpy3DParms,
        deps: impl IntoIterator<Item = &'a GraphNode<'a>>,
    ) -> MemcpyNode {
        let deps = collect_dependencies(deps);

        let mut node = null_mut();
        driver!(hcGraphAddMemcpyNode(
            &mut node,
            self.as_raw(),
            deps.as_ptr(),
            deps.len(),
            params,
            // null_mut(),
        ));
        MemcpyNode(node, PhantomData)
    }
}

#[cfg(test)]
mod test {
    use crate::{DevByte, Device, Graph, VirMem, memcpy_d2h, memcpy_h2d};

    #[test]
    fn test_d2d() {
        if let Err(crate::NoDevice) = crate::init() {
            return;
        }

        Device::new(0).context().apply(|ctx| {
            let mut src = ctx.malloc::<u8>(2 << 10);
            let mut dst = ctx.malloc::<u8>(2 << 10);

            let graph = Graph::new();
            graph.add_memcpy_d2d(&mut dst, &src, &[]);
            test_memcpy_in_graph(&ctx.dev(), &graph, &mut dst, &mut src, 0..u64::MAX);
        })
    }

    #[test]
    fn test_vm() {
        if let Err(crate::NoDevice) = crate::init() {
            return;
        }

        let dev = Device::new(0);
        let prop = dev.mem_prop();
        let minium = prop.granularity_minimum();

        let mut dst_vir = VirMem::new(minium, 0);
        let mut src_vir = VirMem::new(minium, 0);
        let dst = dst_vir.map(0, prop.create(minium));
        let src = src_vir.map(0, prop.create(minium));

        let graph = Graph::new();
        // 虚存不能直接传入 memcpy node，当时必须是已映射状态
        graph.add_memcpy_d2d(dst, src, &[]);

        let phy0 = prop.create(minium);
        let phy1 = prop.create(minium);
        let phy2 = prop.create(minium);
        let phy3 = prop.create(minium);

        let _ = dst_vir.unmap(0);
        let _ = src_vir.unmap(0);
        let dst = dst_vir.map(0, phy0);
        let src = src_vir.map(0, phy1);
        test_memcpy_in_graph(&dev, &graph, dst, src, 0..);

        let _phy0 = dst_vir.unmap(0);
        let _phy1 = src_vir.unmap(0);
        let dst = dst_vir.map(0, phy2);
        let src = src_vir.map(0, phy3);
        test_memcpy_in_graph(&dev, &graph, dst, src, (0..u64::MAX).rev());
    }

    fn test_memcpy_in_graph(
        dev: &Device,
        graph: &Graph,
        dst: &mut [DevByte],
        src: &mut [DevByte],
        origin: impl IntoIterator<Item = u64>,
    ) {
        assert_eq!(dst.len(), src.len());
        let origin = origin
            .into_iter()
            .take(dst.len() / size_of::<u64>())
            .collect::<Box<_>>();
        dev.context().apply(|ctx| {
            memcpy_h2d(src, &origin);

            ctx.stream().launch_graph(&ctx.instantiate(graph));

            let mut host = vec![0u64; origin.len()];
            memcpy_d2h(&mut host, dst);
            assert_eq!(host, &*origin)
        })
    }
}
