pub use self::selector::{CFSelector, CFSelectorSegment};
use std::ops::{Index, IndexMut};

mod selector;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlFlowElement {
    Block {
        content: Vec<ControlFlowElement>,
    },
    If {
        condition: Box<ControlFlowElement>,
        on_success: Vec<ControlFlowElement>,
        on_failure: Vec<ControlFlowElement>,
    },
    Loop {
        content: Vec<ControlFlowElement>,
    },
    BasicBlock {
        id: usize,
    },
}

impl ControlFlowElement {
    pub fn new_node(index: usize) -> Self {
        Self::BasicBlock { id: index }
    }
    pub fn new_block(content: Vec<ControlFlowElement>) -> Self {
        Self::Block { content }
    }
    pub fn first_basic_block_id(&self) -> usize {
        match self {
            ControlFlowElement::Block { content } => content[0].first_basic_block_id(),
            ControlFlowElement::If { condition, .. } => condition.first_basic_block_id(),
            ControlFlowElement::Loop { content } => content[0].first_basic_block_id(),
            ControlFlowElement::BasicBlock { id: node_id } => *node_id,
        }
    }
    pub fn first_basic_block_selector(&self) -> CFSelector {
        match self {
            ControlFlowElement::Block { content } | ControlFlowElement::Loop { content } => {
                let mut result = content[0].first_basic_block_selector();
                result.push_front(CFSelectorSegment::ContentAtIndex(0));
                result
            }
            ControlFlowElement::If { .. } => {
                CFSelector::from_segment(CFSelectorSegment::IfCondition)
            }
            ControlFlowElement::BasicBlock { .. } => CFSelector::new_empty(),
        }
    }
    fn select_mut_by_segment(&mut self, segment: CFSelectorSegment) -> &mut ControlFlowElement {
        match (self, segment) {
            (ControlFlowElement::Block { content }, CFSelectorSegment::ContentAtIndex(index))
            | (ControlFlowElement::Loop { content }, CFSelectorSegment::ContentAtIndex(index)) => {
                &mut content[index]
            }
            (ControlFlowElement::If { condition, .. }, CFSelectorSegment::IfCondition) => {
                condition.as_mut()
            }
            (
                ControlFlowElement::If { on_success, .. },
                CFSelectorSegment::IndexInSuccess(index),
            ) => &mut on_success[index],
            (
                ControlFlowElement::If { on_failure, .. },
                CFSelectorSegment::IndexInFailure(index),
            ) => &mut on_failure[index],
            (
                _,
                CFSelectorSegment::IfCondition
                | CFSelectorSegment::IndexInFailure(_)
                | CFSelectorSegment::IndexInSuccess(_),
            ) => unreachable!(),
            (ControlFlowElement::If { .. }, CFSelectorSegment::ContentAtIndex(_)) => unreachable!(),
            (ControlFlowElement::BasicBlock { .. }, _) => unreachable!(),
        }
    }
    fn select_by_segment(&self, segment: CFSelectorSegment) -> Option<&ControlFlowElement> {
        match (self, segment) {
            (ControlFlowElement::Block { content }, CFSelectorSegment::ContentAtIndex(index))
            | (ControlFlowElement::Loop { content }, CFSelectorSegment::ContentAtIndex(index)) => {
                content.get(index)
            }
            (ControlFlowElement::If { condition, .. }, CFSelectorSegment::IfCondition) => {
                Some(condition.as_ref())
            }
            (
                ControlFlowElement::If { on_success, .. },
                CFSelectorSegment::IndexInSuccess(index),
            ) => on_success.get(index),
            (
                ControlFlowElement::If { on_failure, .. },
                CFSelectorSegment::IndexInFailure(index),
            ) => on_failure.get(index),
            (
                _,
                CFSelectorSegment::IfCondition
                | CFSelectorSegment::IndexInFailure(_)
                | CFSelectorSegment::IndexInSuccess(_),
            ) => unreachable!(),
            (ControlFlowElement::If { .. }, CFSelectorSegment::ContentAtIndex(_)) => unreachable!(),
            (ControlFlowElement::BasicBlock { .. }, _) => unreachable!(),
        }
    }
    pub fn unwrap_node(&self) -> usize {
        if let Self::BasicBlock { id: node_id } = self {
            *node_id
        } else {
            unreachable!()
        }
    }
    pub fn unwrap_content_mut(&mut self) -> &mut Vec<ControlFlowElement> {
        match self {
            Self::Block { content, .. } | Self::Loop { content, .. } => content,
            _ => unreachable!(),
        }
    }
    fn exists(&self, element: &CFSelector) -> bool {
        if let Some((first, rest)) = element.clone().split_first() {
            let subcontent = match (self, first) {
                (
                    ControlFlowElement::Block { content } | ControlFlowElement::Loop { content },
                    CFSelectorSegment::ContentAtIndex(i),
                ) => content.get(i),
                (ControlFlowElement::BasicBlock { .. }, _) => return false,
                (ControlFlowElement::If { .. }, CFSelectorSegment::IfCondition) => {
                    return rest.is_empty()
                }
                (
                    ControlFlowElement::If { on_success, .. },
                    CFSelectorSegment::IndexInSuccess(i),
                ) => on_success.get(i),
                (
                    ControlFlowElement::If { on_failure, .. },
                    CFSelectorSegment::IndexInFailure(i),
                ) => on_failure.get(i),
                _ => unreachable!(),
            };
            if let Some(subcontent) = subcontent {
                subcontent.exists(&rest)
            } else {
                false
            }
        } else {
            true
        }
    }
    pub fn next_element_sibling(&self, element: &CFSelector) -> Option<CFSelector> {
        let mut result = element.clone();
        let back = result.pop_back().unwrap();
        let back = match back {
            CFSelectorSegment::ContentAtIndex(i) => CFSelectorSegment::ContentAtIndex(i + 1),
            CFSelectorSegment::IndexInSuccess(i) => CFSelectorSegment::IndexInSuccess(i + 1),
            CFSelectorSegment::IndexInFailure(i) => CFSelectorSegment::IndexInFailure(i + 1),
            CFSelectorSegment::IfCondition => return None, //maybe: return self.next_element_sibling(&element.parent()?),
        };
        result.push_back(back);
        if self.exists(&result) {
            Some(result)
        } else {
            None
        }
    }
    pub fn find_node(&self, node_id: usize) -> Option<CFSelector> {
        match self {
            ControlFlowElement::BasicBlock { id: self_node_id } if *self_node_id == node_id => {
                Some(CFSelector::new_empty())
            }
            ControlFlowElement::BasicBlock { .. } => None,
            ControlFlowElement::Block { content } | ControlFlowElement::Loop { content } => {
                for (i, c) in content.iter().enumerate() {
                    if let Some(mut subresult) = c.find_node(node_id) {
                        subresult.push_front(CFSelectorSegment::ContentAtIndex(i));
                        return Some(subresult);
                    }
                }
                None
            }
            ControlFlowElement::If {
                condition,
                on_success,
                on_failure,
            } => {
                if let Some(mut subresult) = condition.find_node(node_id) {
                    subresult.push_front(CFSelectorSegment::IfCondition);
                    return Some(subresult);
                }
                for (i, c) in on_success.iter().enumerate() {
                    if let Some(mut subresult) = c.find_node(node_id) {
                        subresult.push_front(CFSelectorSegment::IndexInSuccess(i));
                        return Some(subresult);
                    }
                }
                for (i, c) in on_failure.iter().enumerate() {
                    if let Some(mut subresult) = c.find_node(node_id) {
                        subresult.push_front(CFSelectorSegment::IndexInFailure(i));
                        return Some(subresult);
                    }
                }
                None
            }
        }
    }
    pub fn replace(&mut self, selector: &CFSelector, to_element: ControlFlowElement) {
        assert!(!selector.is_empty());
        let (parent_selector, last_segment) = selector.clone().split_last().unwrap();
        let parent = &mut self[&parent_selector];
        match (parent, last_segment) {
            (
                ControlFlowElement::Block { content } | ControlFlowElement::Loop { content },
                CFSelectorSegment::ContentAtIndex(index),
            ) => {
                content[index] = to_element;
            }
            (ControlFlowElement::If { condition, .. }, CFSelectorSegment::IfCondition) => {
                *condition = Box::new(to_element);
            }
            (
                ControlFlowElement::If { on_success, .. },
                CFSelectorSegment::IndexInSuccess(index),
            ) => on_success[index] = to_element,
            (
                ControlFlowElement::If { on_failure, .. },
                CFSelectorSegment::IndexInFailure(index),
            ) => on_failure[index] = to_element,
            (ControlFlowElement::If { .. }, CFSelectorSegment::ContentAtIndex(_)) => unreachable!(),
            (ControlFlowElement::Block { .. }, _) => unreachable!(),
            (ControlFlowElement::Loop { .. }, _) => unreachable!(),
            (ControlFlowElement::BasicBlock { .. }, _) => unreachable!(),
        }
    }
    pub fn remove(&mut self, element: &CFSelector) -> ControlFlowElement {
        assert!(!element.is_empty());
        let (parent_selector, last_segment) = element.clone().split_last().unwrap();
        let parent = &mut self[&parent_selector];
        match (parent, last_segment) {
            (ControlFlowElement::Block { content }, CFSelectorSegment::ContentAtIndex(i)) => {
                content.remove(i)
            }
            (ControlFlowElement::If { on_success, .. }, CFSelectorSegment::IndexInSuccess(i)) => {
                on_success.remove(i)
            }
            (ControlFlowElement::If { on_failure, .. }, CFSelectorSegment::IndexInFailure(i)) => {
                on_failure.remove(i)
            }
            (ControlFlowElement::Loop { content }, CFSelectorSegment::ContentAtIndex(i)) => {
                content.remove(i)
            }
            (_, CFSelectorSegment::IfCondition) => panic!("You should not remove the condition of an if statement, maybe try remove the whole if statement instead?"),
            (ControlFlowElement::Block { .. }, _) => unreachable!(),
            (ControlFlowElement::If { .. }, _) => unreachable!(),
            (ControlFlowElement::Loop { .. }, _) => unreachable!(),
            (ControlFlowElement::BasicBlock { .. }, _) => unreachable!(),
        }
    }
    pub fn get(&self, index: &CFSelector) -> Option<&ControlFlowElement> {
        if index.is_empty() {
            Some(self)
        } else {
            let (first, rest) = index.clone().split_first()?;
            let first_result = self.select_by_segment(first);
            first_result?.get(&rest)
        }
    }
    pub fn block_content(&self) -> Option<&Vec<ControlFlowElement>> {
        if let ControlFlowElement::Block { content } = self {
            Some(content)
        } else {
            None
        }
    }
}

impl IndexMut<&CFSelector> for ControlFlowElement {
    fn index_mut(&mut self, index: &CFSelector) -> &mut Self::Output {
        if index.is_empty() {
            self
        } else {
            let (first, rest) = index.clone().split_first().unwrap();
            let first_result = self.select_mut_by_segment(first);
            first_result.index_mut(&rest)
        }
    }
}

impl Index<&CFSelector> for ControlFlowElement {
    type Output = ControlFlowElement;

    fn index(&self, index: &CFSelector) -> &Self::Output {
        self.get(index).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use itertools::Itertools;

    use super::*;

    #[test]
    fn test_find_node() {
        let current_result = (0..5)
            .map(|it| ControlFlowElement::BasicBlock { id: it })
            .collect_vec();

        let content = ControlFlowElement::Block {
            content: current_result,
        };
        let selector = content.find_node(0);
        assert_eq!(selector.unwrap(), CFSelector::from_str("0").unwrap());
        let selector = content.find_node(1);
        assert_eq!(selector.unwrap(), CFSelector::from_str("1").unwrap());
        let content = ControlFlowElement::Block {
            content: vec![
                ControlFlowElement::BasicBlock { id: 0 },
                ControlFlowElement::If {
                    condition: Box::new(ControlFlowElement::BasicBlock { id: 1 }),
                    on_success: Vec::new(),
                    on_failure: Vec::new(),
                },
                ControlFlowElement::BasicBlock { id: 2 },
            ],
        };
        let selector = content.find_node(1);
        assert_eq!(
            selector.unwrap(),
            CFSelector::from_str("1/if_condition").unwrap()
        );

        let content = ControlFlowElement::Block {
            content: vec![
                ControlFlowElement::BasicBlock { id: 0 },
                ControlFlowElement::If {
                    condition: Box::new(ControlFlowElement::BasicBlock { id: 1 }),
                    on_success: vec![ControlFlowElement::BasicBlock { id: 2 }],
                    on_failure: Vec::new(),
                },
                ControlFlowElement::BasicBlock { id: 3 },
            ],
        };
        let selector = content.find_node(1);
        assert_eq!(
            selector.unwrap(),
            CFSelector::from_str("1/if_condition").unwrap()
        );
        let selector = content.find_node(2);
        assert_eq!(
            selector.unwrap(),
            CFSelector::from_str("1/success->0").unwrap()
        );
        let selector = content.find_node(3);
        assert_eq!(selector.unwrap(), CFSelector::from_str("2").unwrap());
    }
}
