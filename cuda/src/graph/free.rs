﻿use super::{Graph, GraphNode, MemFreeNode, collect_dependencies};
use crate::VirByte;
use context_spore::AsRaw;
use std::{marker::PhantomData, ptr::null_mut};

impl Graph {
    pub fn free<'a>(
        &self,
        ptr: *const VirByte,
        deps: impl IntoIterator<Item = &'a GraphNode<'a>>,
    ) -> MemFreeNode {
        let deps = collect_dependencies(deps);

        let mut node = null_mut();
        driver!(hcGraphAddMemFreeNode(
            &mut node,
            self.as_raw(),
            deps.as_ptr(),
            deps.len(),
            ptr as _,
        ));
        MemFreeNode(node, PhantomData)
    }

    pub fn add_free_node<'a>(
        &self,
        _node: &MemFreeNode,
        _deps: impl IntoIterator<Item = &'a GraphNode<'a>>,
    ) -> MemFreeNode {
        todo!()
    }
}
