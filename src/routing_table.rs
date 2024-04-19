use std::cmp::Ordering;
use crate::node::NodeInfo;

#[derive(Debug, Clone)]
pub struct KBucket<T: Clone, NodeInfo> {
    pub value: T,
    pub info: NodeInfo,
    pub left: Option<Box<KBucket<T, NodeInfo>>>,
    pub right: Option<Box<KBucket<T, NodeInfo>>>,
}

impl<T: Clone + Ord> KBucket<T, NodeInfo> {
    pub fn new(value: T, info: NodeInfo) -> Self {
        KBucket {
            value,
            info,
            left: None,
            right: None,
        }
    }

    pub fn insert(&mut self, value: T, info:NodeInfo) {
        match value.cmp(&self.value) {
            Ordering::Less => {
                if let Some(ref mut left) = self.left {
                    left.insert(value, info);
                } else {
                    self.left = Some(Box::new(KBucket::new(value, info)));
                }
            }
            Ordering::Greater => {
                if let Some(ref mut right) = self.right {
                    right.insert(value, info);
                } else {
                    self.right = Some(Box::new(KBucket::new(value, info)));
                }
            }
            Ordering::Equal => {}
        }
    }

    pub fn contains(&self, value: &T) -> bool {
        match value.cmp(&self.value) {
            Ordering::Less => self.left.as_ref().map_or(false, |n| n.contains(value)),
            Ordering::Greater => self.right.as_ref().map_or(false, |n| n.contains(value)),
            Ordering::Equal => true,
        }
    }
    pub fn clone_bucket(&self) -> Box<KBucket<T, NodeInfo>> {
        let mut cloned_bucket = KBucket::new(self.value.clone(), self.info.clone());
        cloned_bucket.left = self.left.as_ref().map(|left| left.clone_bucket());
        cloned_bucket.right = self.right.as_ref().map(|right| right.clone_bucket());
        Box::new(cloned_bucket)
    }
}

#[derive(Debug, Clone)]
pub struct RoutingTable<T: Clone, NodeInfo> {
    pub root: Option<Box<KBucket<T, NodeInfo>>>,
}

impl<T: Clone + Ord> RoutingTable<T, NodeInfo> {
    pub fn new() -> Self {
        RoutingTable { root: None }
    }

    pub fn insert(&mut self, value: T, info:NodeInfo) {
        if let Some(ref mut root) = self.root {
            root.insert(value, info);
        } else {
            self.root = Some(Box::new(KBucket::new(value, info)));
        }
    }

    pub fn contains(&self, value: &T) -> bool {
        self.root.as_ref().map_or(false, |root| root.contains(value))
    }

    pub fn clone(&self) -> Self {
        Self {
            root: self.root.as_ref().map(|bucket| bucket.clone_bucket()),
        }
    }
}