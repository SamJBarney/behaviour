use std::{marker::Tuple, rc::Rc};
use std::rc::Weak;

use crate::registry::Identifier;
use super::context::BehaviourContext;

type VecType = u32;
const NODE_SIZE: usize = (u64::BITS / VecType::BITS) as usize;
const ID_MASK: u32 = 0xF000;


const SEQUENCE_ID: u8 = 1;
const FALLBACK_ID: u8 = 2;
const PARALLEL_ID: u8 = 3;
const DECORATOR_ID: u8 = 4;
const EXECUTOR_ID: u8 = 5;

pub enum BehaviourNode {
    Root(Box<BehaviourNode>),
    Sequence {
        children: Vec<BehaviourNode>,
    },
    Fallback {
        children: Vec<BehaviourNode>,
    },
    Parallel {
        children: Vec<BehaviourNode>,
    },
    Decorator {
        name: Identifier,
        child: Box<BehaviourNode>,
    },
    Executor(Identifier),
}

impl BehaviourNode {
    pub fn compile<Calltype: Tuple>(
        self,
        ctx: Weak<BehaviourContext<Calltype>>,
    ) -> Result<BehaviourTree<Calltype>, TreeCompilationError> {
        match self {
            BehaviourNode::Root(child) => child.compile_inner(ctx),
            _ => Err(TreeCompilationError::InitialNonRootNode),
        }
    }
    fn compile_inner<Calltype: Tuple>(
        self,
        context: Weak<BehaviourContext<Calltype>>,
    ) -> Result<BehaviourTree<Calltype>, TreeCompilationError> {
        let ctx_wrapped = context.upgrade();
        if ctx_wrapped.is_none() {
            return Err(TreeCompilationError::NonExistentContext);
        }
        let ctx = ctx_wrapped.unwrap();
        let mut nodes = Vec::new();
        let mut code = Vec::new();
        let mut node_offset: usize = 0;
        let mut node_count = 0;
        nodes.push(self);

        while nodes.len() > 0 {
            let node = nodes.pop().unwrap();

            match node {
                Self::Root(_) => return Err(TreeCompilationError::RootNodeInTree),
                Self::Sequence { mut children } => {
                    let child_count = children.len() as u32;
                    if children.len() > 0 {
                        if child_count & !ID_MASK != child_count {
                            return Err(TreeCompilationError::TooManyChildNodes);
                        }
                        code.push(((SEQUENCE_ID as VecType) << 24) | child_count);
                        node_offset += NODE_SIZE;

                        let child_offset = (node_offset + nodes.len() * NODE_SIZE) as u32;
                        code.push(child_offset);

                        nodes.append(&mut children);

                        node_count += 1;
                    }
                }
                Self::Fallback { mut children } => {
                    let child_count = children.len() as u32;
                    if children.len() > 0 {
                        if child_count & !ID_MASK != child_count {
                            return Err(TreeCompilationError::TooManyChildNodes);
                        }
                        code.push(((FALLBACK_ID as VecType) << 24) | child_count);
                        node_offset += NODE_SIZE;

                        let child_offset = (node_offset + nodes.len() * NODE_SIZE) as u32;
                        code.push(child_offset);

                        nodes.append(&mut children);

                        nodes.append(&mut children);

                        node_count += 1;
                    }
                }
                Self::Parallel { mut children } => {
                    let child_count = children.len() as u32;
                    if children.len() > 0 {
                        if child_count & !ID_MASK != child_count {
                            return Err(TreeCompilationError::TooManyChildNodes);
                        }
                        code.push(((PARALLEL_ID as VecType) << 24) | child_count);
                        node_offset += NODE_SIZE;

                        let child_offset = (node_offset + nodes.len() * NODE_SIZE) as u32;
                        code.push(child_offset);

                        nodes.append(&mut children);

                        node_count += 1;
                    }
                }
                Self::Decorator { name, child } => {
                    if let Some(handle) = ctx.get_decorator_handle(&name) {
                        let handle_value = handle.value();
                        let masked_handle = (handle_value as u32) & !ID_MASK;
                        if masked_handle as usize != handle_value {
                            return Err(TreeCompilationError::UnencodableRegistryHandle { id: name, registry_index: handle_value});
                        }
                        node_offset += NODE_SIZE;
                        
                        code.push(masked_handle | ((DECORATOR_ID as u32) << 24));

                        let child_offset = (node_offset + nodes.len() * NODE_SIZE) as u32;
                        println!("CHILD_OFFSET: {:?}", child_offset);
                        code.push(child_offset);

                        nodes.push(Box::into_inner(child));

                        node_count += 1;
                    } else {
                        return Err(TreeCompilationError::UnknownDecorator(name));
                    }
                }
                Self::Executor(id) => {
                    if let Some(handle) = ctx.get_executor_handle(&id) {
                        let handle_value = handle.value();
                        let masked_handle = (handle_value as u32) & !ID_MASK;
                        if masked_handle as usize != handle_value {
                            return Err(TreeCompilationError::UnencodableRegistryHandle { id, registry_index: handle_value});
                        }
                        node_offset += NODE_SIZE;
                        
                        code.push(masked_handle | ((EXECUTOR_ID as u32) << 24));
                        code.push(0);

                        node_count += 1;
                    } else {
                        return Err(TreeCompilationError::UnknownExecutor(id));
                    }
                }
            }
        }
        if node_count > 0 {
            Ok(BehaviourTree {
                code,
                context: ctx,
                node_count,
            })
        } else {
            Err(TreeCompilationError::NoNodes)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TreeCompilationError {
    NoNodes,
    InitialNonRootNode,
    RootNodeInTree,
    UnknownDecorator(Identifier),
    UnknownExecutor(Identifier),
    UnencodableRegistryHandle{id: Identifier, registry_index: usize},
    TooManyChildNodes,
    NonExistentContext,
}

#[derive(Debug)]
pub struct BehaviourTree<CallType: Tuple> {
    code: Vec<VecType>,
    context: Rc<BehaviourContext<CallType>>,
    node_count: usize,
}

impl<Calltype: Tuple> BehaviourTree<Calltype> {
    pub fn code(&self) -> &Vec<VecType> {
        &self.code
    }
    
    pub fn context(&self) -> &BehaviourContext<Calltype> {
        self.context.as_ref()
    }
    
    pub fn node_count(&self) -> usize {
        self.node_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod behaviour_node {
        use std::rc::Rc;

        use crate::behavior::context::BehaviourContext;

        use super::{BehaviourNode as Subject, *};

        pub mod test_funcs {
            use crate::behavior::state::TreeResult;

            pub fn executor(_: ()) -> TreeResult {
                TreeResult::Success
            }

            pub fn decorator(_: TreeResult, _1: ()) -> TreeResult {
                TreeResult::Success
            }
        }

        #[test]
        fn compile_fails_first_node_control_sequence() {
            let ctx: Rc<BehaviourContext<()>> = Rc::new(BehaviourContext::new());
            let subject = Subject::Sequence {
                children: Vec::new(),
            };
            assert!(subject
                .compile::<()>(Rc::downgrade(&ctx))
                .is_err_and(|err| err == TreeCompilationError::InitialNonRootNode));
        }

        #[test]
        fn compile_fails_first_node_control_fallback() {
            let ctx: Rc<BehaviourContext<()>> = Rc::new(BehaviourContext::new());
            let subject = Subject::Fallback {
                children: Vec::new(),
            };
            assert!(subject.compile::<()>(Rc::downgrade(&ctx)).is_err());
        }

        #[test]
        fn compile_fails_first_node_control_parallel() {
            let ctx: Rc<BehaviourContext<()>> = Rc::new(BehaviourContext::new());
            let subject = Subject::Parallel {
                children: Vec::new(),
            };
            assert!(subject
                .compile::<()>(Rc::downgrade(&ctx))
                .is_err_and(|err| err == TreeCompilationError::InitialNonRootNode));
        }

        #[test]
        fn compile_fails_first_node_control_decorator() {
            let ctx: Rc<BehaviourContext<()>> = Rc::new(BehaviourContext::new());
            let subject = Subject::Decorator {
                name: "test".into(),
                child: Box::new(Subject::Executor("".into())),
            };
            assert!(subject
                .compile::<()>(Rc::downgrade(&ctx))
                .is_err_and(|err| err == TreeCompilationError::InitialNonRootNode));
        }

        #[test]
        fn compile_fails_first_node_execute() {
            let ctx: Rc<BehaviourContext<()>> = Rc::new(BehaviourContext::new());
            let subject = Subject::Executor("".into());
            assert!(subject
                .compile::<()>(Rc::downgrade(&ctx))
                .is_err_and(|err| err == TreeCompilationError::InitialNonRootNode));
        }

        #[test]
        fn compile_fails_no_nodes_control_sequence() {
            let ctx: Rc<BehaviourContext<()>> = Rc::new(BehaviourContext::new());
            let subject = Subject::Root(Box::new(Subject::Sequence {
                children: Vec::new(),
            }));
            assert!(subject
                .compile::<()>(Rc::downgrade(&ctx))
                .is_err_and(|err| err == TreeCompilationError::NoNodes));
        }

        #[test]
        fn compile_fails_no_nodes_control_fallback() {
            let ctx: Rc<BehaviourContext<()>> = Rc::new(BehaviourContext::new());
            let subject = Subject::Root(Box::new(Subject::Fallback {
                children: Vec::new(),
            }));
            assert!(subject
                .compile::<()>(Rc::downgrade(&ctx))
                .is_err_and(|err| err == TreeCompilationError::NoNodes));
        }

        #[test]
        fn compile_fails_no_nodes_control_parallel() {
            let ctx: Rc<BehaviourContext<()>> = Rc::new(BehaviourContext::new());
            let subject = Subject::Root(Box::new(Subject::Parallel {
                children: Vec::new(),
            }));
            assert!(subject
                .compile::<()>(Rc::downgrade(&ctx))
                .is_err_and(|err| err == TreeCompilationError::NoNodes));
        }

        #[test]
        fn compile_fails_unknown_decorator_control_decorator() {
            let ctx: Rc<BehaviourContext<()>> = Rc::new(BehaviourContext::new());
            let subject = Subject::Root(Box::new(Subject::Decorator {
                name: "decorator".into(),
                child: Box::new(Subject::Executor("".into())),
            }));
            assert!(subject.compile::<()>(Rc::downgrade(&ctx)).is_err_and(
                |err| err == TreeCompilationError::UnknownDecorator("decorator".into())
            ));
        }

        #[test]
        fn compile_fails_unknown_handler_executor() {
            let ctx: Rc<BehaviourContext<()>> = Rc::new(BehaviourContext::new());
            let subject = Subject::Root(Box::new(Subject::Executor("executor".into())));
            assert!(subject
                .compile::<()>(Rc::downgrade(&ctx))
                .is_err_and(|err| err == TreeCompilationError::UnknownExecutor("executor".into())));
        }

        #[test]
        fn compile_success_executor() {
            let mut context = BehaviourContext::new();
            context
                .register_executor(&"exec".into(), test_funcs::executor)
                .unwrap();
            let ctx: Rc<BehaviourContext<()>> = Rc::new(context);

            let subject = Subject::Root(Box::new(Subject::Executor("exec".into())));
            let res = subject.compile(Rc::downgrade(&ctx));
            assert!(res.is_ok());

            if let Ok(tree) = res {
                assert_eq!(tree.node_count, 1);
                assert_eq!(tree.code, vec![(EXECUTOR_ID as VecType) << 24, 0]);
            }
        }

        #[test]
        fn compile_success_decorator() {
            let mut context = BehaviourContext::new();
            context
                .register_decorator(&"decorate".into(), test_funcs::decorator)
                .unwrap();
            context
                .register_executor(&"exec".into(), test_funcs::executor)
                .unwrap();
            let ctx: Rc<BehaviourContext<()>> = Rc::new(context);

            let subject = Subject::Root(Box::new(Subject::Decorator {
                name: "decorate".into(),
                child: Box::new(Subject::Executor("exec".into())),
            }));
            let res = subject.compile(Rc::downgrade(&ctx));
            assert!(res.is_ok());
            
            if let Ok(tree) = res {
                assert_eq!(tree.node_count, 2);
                assert_eq!(tree.code, vec![(DECORATOR_ID as VecType) << 24, 2, (EXECUTOR_ID as VecType) << 24, 0]);
            }
        }
        
        #[test]
        fn compile_success_parallel() {
            let mut context = BehaviourContext::new();
            context
                .register_executor(&"exec".into(), test_funcs::executor)
                .unwrap();
            let ctx: Rc<BehaviourContext<()>> = Rc::new(context);

            let subject = Subject::Root(Box::new(Subject::Parallel {
                children: vec![Subject::Executor("exec".into())],
            }));
            let res = subject.compile(Rc::downgrade(&ctx));
            assert!(res.is_ok());
            
            if let Ok(tree) = res {
                assert_eq!(tree.node_count, 2);
                assert_eq!(tree.code, vec![((PARALLEL_ID as VecType) << 24) | 1, 2, (EXECUTOR_ID as VecType) << 24, 0]);
            }
        }
        
        #[test]
        fn compile_success_parallel_multiple_children() {
            let mut context = BehaviourContext::new();
            context
                .register_executor(&"exec".into(), test_funcs::executor)
                .unwrap();
            let ctx: Rc<BehaviourContext<()>> = Rc::new(context);

            let subject = Subject::Root(Box::new(Subject::Parallel {
                children: vec![Subject::Executor("exec".into()), Subject::Executor("exec".into())],
            }));
            let res = subject.compile(Rc::downgrade(&ctx));
            assert!(res.is_ok());
            
            if let Ok(tree) = res {
                assert_eq!(tree.node_count, 3);
                assert_eq!(tree.code, vec![((PARALLEL_ID as VecType) << 24) | 2, 2, (EXECUTOR_ID as VecType) << 24, 0, (EXECUTOR_ID as VecType) << 24, 0]);
            }
        }
        
        #[test]
        fn compile_success_fallback() {
            let mut context = BehaviourContext::new();
            context
                .register_executor(&"exec".into(), test_funcs::executor)
                .unwrap();
            let ctx: Rc<BehaviourContext<()>> = Rc::new(context);

            let subject = Subject::Root(Box::new(Subject::Fallback {
                children: vec![Subject::Executor("exec".into())],
            }));
            let res = subject.compile(Rc::downgrade(&ctx));
            assert!(res.is_ok());
            
            if let Ok(tree) = res {
                assert_eq!(tree.node_count, 2);
                assert_eq!(tree.code, vec![((FALLBACK_ID as VecType) << 24) | 1, 2, (EXECUTOR_ID as VecType) << 24, 0]);
            }
        }
        
        #[test]
        fn compile_success_fallback_multiple_children() {
            let mut context = BehaviourContext::new();
            context
                .register_executor(&"exec".into(), test_funcs::executor)
                .unwrap();
            let ctx: Rc<BehaviourContext<()>> = Rc::new(context);

            let subject = Subject::Root(Box::new(Subject::Fallback {
                children: vec![Subject::Executor("exec".into()), Subject::Executor("exec".into())],
            }));
            let res = subject.compile(Rc::downgrade(&ctx));
            assert!(res.is_ok());
            
            if let Ok(tree) = res {
                assert_eq!(tree.node_count, 3);
                assert_eq!(tree.code, vec![((FALLBACK_ID as VecType) << 24) | 2, 2, (EXECUTOR_ID as VecType) << 24, 0, (EXECUTOR_ID as VecType) << 24, 0]);
            }
        }
        
        #[test]
        fn compile_success_sequence() {
            let mut context = BehaviourContext::new();
            context
                .register_decorator(&"decorate".into(), test_funcs::decorator)
                .unwrap();
            context
                .register_executor(&"exec".into(), test_funcs::executor)
                .unwrap();
            let ctx: Rc<BehaviourContext<()>> = Rc::new(context);

            let subject = Subject::Root(Box::new(Subject::Sequence {
                children: vec![Subject::Executor("exec".into())],
            }));
            let res = subject.compile(Rc::downgrade(&ctx));
            assert!(res.is_ok());
            
            if let Ok(tree) = res {
                assert_eq!(tree.node_count, 2);
            }
        }
        
        #[test]
        fn compile_success_sequence_multiple_children() {
            let mut context = BehaviourContext::new();
            context
                .register_decorator(&"decorate".into(), test_funcs::decorator)
                .unwrap();
            context
                .register_executor(&"exec".into(), test_funcs::executor)
                .unwrap();
            let ctx: Rc<BehaviourContext<()>> = Rc::new(context);

            let subject = Subject::Root(Box::new(Subject::Sequence {
                children: vec![Subject::Executor("exec".into()), Subject::Executor("exec".into())],
            }));
            let res = subject.compile(Rc::downgrade(&ctx));
            assert!(res.is_ok());
            
            if let Ok(tree) = res {
                assert_eq!(tree.node_count, 3);
                assert_eq!(tree.code, vec![((SEQUENCE_ID as VecType) << 24) | 2, 2, (EXECUTOR_ID as VecType) << 24, 0, (EXECUTOR_ID as VecType) << 24, 0]);
            }
        }
    }

    mod behaviour_tree {}
}
