use delegate::delegate;
use std::{
    cmp::Ordering, collections::VecDeque, fmt, iter::zip, num::ParseIntError, ops::RangeBounds, str::FromStr,
};

#[derive(Clone, PartialEq, Eq)]
pub enum CFSelectorSegment {
    ContentAtIndex(usize),
    IfCondition,
    IndexInSuccess(usize),
    IndexInFailure(usize),
}

impl fmt::Display for CFSelectorSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CFSelectorSegment::ContentAtIndex(index) => write!(f, "{}", index),
            CFSelectorSegment::IfCondition => write!(f, "if_condition"),
            CFSelectorSegment::IndexInSuccess(index) => write!(f, "success->{}", index),
            CFSelectorSegment::IndexInFailure(index) => write!(f, "failure->{}", index),
        }
    }
}

impl fmt::Debug for CFSelectorSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl FromStr for CFSelectorSegment {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "if_condition" {
            Ok(CFSelectorSegment::IfCondition)
        } else if s.starts_with("success->") {
            let index_str = s.strip_prefix("success->").unwrap();
            let value = index_str.parse()?;
            Ok(CFSelectorSegment::IndexInSuccess(value))
        } else if s.starts_with("failure->") {
            let index_str = s.strip_prefix("failure->").unwrap();
            let value = index_str.parse()?;
            Ok(CFSelectorSegment::IndexInFailure(value))
        } else {
            let value = s.parse()?;
            Ok(CFSelectorSegment::ContentAtIndex(value))
        }
    }
}

impl PartialOrd for CFSelectorSegment {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (CFSelectorSegment::ContentAtIndex(i), CFSelectorSegment::ContentAtIndex(j))
            | (CFSelectorSegment::IndexInSuccess(i), CFSelectorSegment::IndexInSuccess(j))
            | (CFSelectorSegment::IndexInFailure(i), CFSelectorSegment::IndexInFailure(j)) => {
                i.partial_cmp(j)
            }
            (CFSelectorSegment::IfCondition, CFSelectorSegment::IfCondition) => {
                Some(Ordering::Equal)
            }
            (
                CFSelectorSegment::IfCondition,
                CFSelectorSegment::IndexInSuccess(_) | CFSelectorSegment::IndexInFailure(_),
            ) => Some(Ordering::Less),
            (
                CFSelectorSegment::IndexInFailure(_) | CFSelectorSegment::IndexInSuccess(_),
                CFSelectorSegment::IfCondition,
            ) => Some(Ordering::Greater),
            (CFSelectorSegment::IndexInSuccess(_), CFSelectorSegment::IndexInFailure(_)) => {
                Some(Ordering::Less)
            }
            (CFSelectorSegment::IndexInFailure(_), CFSelectorSegment::IndexInSuccess(_)) => {
                Some(Ordering::Greater)
            }
            _ => None,
        }
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct CFSelector(VecDeque<CFSelectorSegment>);

impl fmt::Display for CFSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            write!(f, "()")
        } else {
            write!(
                f,
                "{}",
                self.0
                    .iter()
                    .map(|s| format!("{}", s))
                    .collect::<Vec<_>>()
                    .join("/")
            )
        }
    }
}

impl fmt::Debug for CFSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl FromStr for CFSelector {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut result = VecDeque::new();
        let parts = s.split('/');
        for next_part in parts {
            let segment = CFSelectorSegment::from_str(next_part)?;
            result.push_back(segment);
        }
        Ok(Self(result))
    }
}

impl CFSelector {
    pub(super) fn new_empty() -> Self {
        Self(VecDeque::new())
    }

    pub fn from_segment(segment: CFSelectorSegment) -> Self {
        let mut result = VecDeque::new();
        result.push_back(segment);
        Self(result)
    }

    delegate! {
        to self.0 {
            pub fn is_empty(&self) -> bool;
            pub fn len(&self) -> usize;
            pub fn pop_front(&mut self) -> Option<CFSelectorSegment>;
            pub fn pop_back(&mut self) -> Option<CFSelectorSegment>;
            pub fn front(&self) -> Option<&CFSelectorSegment>;
            pub fn back(&self) -> Option<&CFSelectorSegment>;
            pub fn push_front(&mut self, segment: CFSelectorSegment);
            pub fn push_back(&mut self, segment: CFSelectorSegment);
        }
    }

    pub fn parent(&self) -> Option<CFSelector> {
        let mut result = self.0.clone();
        if result.is_empty() {
            None
        } else {
            result.pop_back();
            Some(CFSelector(result))
        }
    }

    pub(super) fn split_first(mut self) -> Option<(CFSelectorSegment, CFSelector)> {
        let front = self.0.pop_front()?;
        Some((front, self))
    }

    pub(super) fn split_last(mut self) -> Option<(CFSelector, CFSelectorSegment)> {
        let back = self.0.pop_back()?;
        Some((self, back))
    }

    fn is_ancestor_of(&self, other: &CFSelector) -> bool {
        for (from_self, from_other) in zip(&self.0, &other.0) {
            if from_self != from_other {
                return false;
            }
        }
        true
    }

    pub fn is_parent_of(&self, other: &CFSelector) -> bool {
        // different from direct parent relationship, this function consider, for example, `0`.is_parent_of(`0/if_condition/0`)
        let lca = self.lowest_common_ancestor(other);
        let parent_rest = self.range(lca.len()..);
        let mut child_rest = other.range(lca.len()..);
        if !parent_rest.is_empty() || child_rest.is_empty() {
            return false;
        }
        while child_rest.len() > 1 {
            let front = child_rest.pop_front().unwrap();
            if front == CFSelectorSegment::IfCondition {
                return false;
            }
        }
        true
    }

    pub(super) fn merge(&self, other: &CFSelector) -> CFSelector {
        let mut result = self.clone();
        result.0.extend(other.0.clone());
        result
    }

    pub fn is_if_condition(&self) -> bool {
        matches!(self.back(), Some(CFSelectorSegment::IfCondition))
    }

    pub fn lowest_common_ancestor(&self, other: &CFSelector) -> CFSelector {
        let mut result = Self::new_empty();
        for (from_self, from_other) in zip(&self.0, &other.0) {
            if from_self == from_other {
                result.0.push_back(from_self.clone());
            } else {
                break;
            }
        }
        result
    }

    pub fn range<R: RangeBounds<usize>>(&self, range: R) -> Self {
        Self(self.0.range(range).cloned().collect())
    }

    pub fn is_sibling(selector: &CFSelector, last_selector: &CFSelector) -> bool {
        if selector.len() != last_selector.len() {
            false
        } else {
            let shared_part = Self::lowest_common_ancestor(selector, last_selector);
            shared_part.len() == selector.len() - 1
        }
    }

    pub fn block_like_count(&self) -> usize {
        self.0
            .iter()
            .filter(|it| matches!(it, CFSelectorSegment::ContentAtIndex(_)))
            .count()
    }

    pub fn is_after(&self, other: &CFSelector) -> Option<bool> {
        for (from_self, from_other) in self.0.iter().zip(other.0.iter()) {
            if from_other < from_self {
                return Some(false);
            } else if from_other > from_self {
                return Some(true);
            }
        }
        None
    }

    pub fn levels_before(&self, other: &CFSelector) -> Option<usize> {
        if self.is_after(other).unwrap_or(false) {
            return None;
        }
        let shared_part = Self::lowest_common_ancestor(self, other);
        let self_unique_part = self.range(shared_part.len()..);
        Some(
            self_unique_part
                .0
                .into_iter()
                .filter(|it| matches!(it, CFSelectorSegment::ContentAtIndex(_)))
                .count(),
        )
    }
}
