use std::iter::Extend;
use std::ptr;
use std::usize;

#[derive(Clone)]
struct QueueNode {
    value: u32,
    pnode: *mut PriorityNode,
}

impl Default for QueueNode {
    fn default() -> Self {
        Self {
            value: 0,
            pnode: ptr::null_mut(),
        }
    }
}

struct PriorityNode {
    priority: u32,
    bounds: [usize; 2],
}

pub struct RangePriorityQueue {
    queue: Vec<QueueNode>,
    head: usize,
    len: usize,
    indicies: Vec<usize>,
}

impl RangePriorityQueue {

    pub fn new(range: usize) -> Self {
        Self {
            queue: vec![QueueNode::default(); range],
            head: 0,
            len: 0,
            indicies: vec![usize::MAX; range],
        }
    }
    
    /**
     * Adds a number to the queue. If the number is already in the queue, its priority will be incremented.
     */
    pub fn add(&mut self, n: u32) {
        let range = self.queue.len();
        let index_add = |a, b| { Self::index_add(range, a, b) };

        match self.index_of(n) {
            Some(mut i) => {
                let swap_index = {
                    let pnode = unsafe { &*self.queue[i].pnode };
                    let left = pnode.bounds[0];
                    if left != i { Some(left) } else { None }
                };
                let node_is_shared = if let Some(swap_index) = swap_index {
                    self.queue.swap(swap_index, i);
                    self.indicies.swap(n as usize, self.queue[i].value as usize);
                    i = swap_index;
                    true
                } else {
                    let pnode = unsafe { &*self.queue[i].pnode };
                    Self::pnode_ref_count(range, pnode) > 1
                };
                if node_is_shared {
                    let right_index = index_add(i, 1);
                    let pnode = unsafe { &mut *self.queue[right_index].pnode };
                    let l = &mut pnode.bounds[0];
                    *l = index_add(*l, 1);
                }
                if i != self.head && self.join_left_if_step_up(i) {
                } else if node_is_shared {
                    let priority = unsafe { &*self.queue[i].pnode }.priority + 1;
                    let pnode = Box::into_raw(Box::new(PriorityNode {
                        priority: priority,
                        bounds: [i, i],
                    }));
                    self.queue[i].pnode = pnode;
                } else {
                    unsafe { &mut *self.queue[i].pnode }.priority += 1;
                }
            },
            None => { // n is not in queue
                let (index, pnode) = if self.len == 0 {
                    let node = Box::into_raw(Box::new(PriorityNode {
                        priority: 0,
                        bounds: [self.head, self.head],
                    }));
                    (self.head, node)
                } else {
                    let last_index = index_add(self.head, self.len - 1);
                    let mut pnode = unsafe { &mut *self.queue[last_index].pnode };
                    if pnode.priority == 0 {
                        let index = {
                            let r = &mut pnode.bounds[1];
                            *r = index_add(*r, 1);
                            *r
                        };
                        let pnode: *mut _ = pnode;
                        (index, pnode)
                    } else {
                        let i = index_add(self.head, self.len);
                        let pnode = Box::into_raw(Box::new(PriorityNode {
                            priority: 0,
                            bounds: [i, i],
                        }));
                        (i, pnode)
                    }
                };
                self.queue[index] = QueueNode { value: n, pnode };
                self.indicies[n as usize] = index;
                self.len += 1;
            }
        }
    }

    pub fn pop(&mut self) -> Option<u32> {
        if self.len == 0 {
            return None
        }
        let n;
        {
            let node = &mut self.queue[self.head];
            n = node.value;
            let pnode = unsafe { &mut *node.pnode };
            if pnode.bounds[0] == pnode.bounds[1] {
                node.value = 0;
                unsafe { Box::from_raw(node.pnode) };
                node.pnode = ptr::null_mut();
            } else {
                pnode.bounds[0] += 1;
            }
        }
        self.indicies[n as usize] = usize::MAX;
        self.head = Self::index_add(self.range(), self.head, 1);
        self.len -= 1;
        Some(n)
    }

    #[inline]
    fn range(&self) -> usize {
        self.queue.len()
    }

    fn index_of(&self, n: u32) -> Option<usize> {
        let i = self.indicies[n as usize];
        match i {
            usize::MAX => None,
            _ => Some(i),
        }
    }

    fn pnode_ref_count(range: usize, node: &PriorityNode) -> u32 {
        Self::index_subtract(range, node.bounds[1], node.bounds[0]) as u32
    }

/*
    unsafe fn node_at(&self, i: usize) -> &Node {
        &*self.queue[i].1
    }

    unsafe fn node_at_mut(&mut self, i: usize) -> &mut Node {
        &mut *self.queue[i].as_mut().unwrap().1.get()
    }

*/
    fn index_add(range: usize, a: usize, b: usize) -> usize {
        (a + b) % range
    }

    fn index_subtract(range: usize, a: usize, b: usize) -> usize {
        if a >= b {
            a - b
        } else {
            range - (b - a)
        }
    }

    fn join_left_if_step_up(&mut self, i: usize) -> bool {
        let len = self.queue.len();
        let higher = Self::index_subtract(len, i, 1);
        let pnode = unsafe { &*self.queue[i].pnode };
        let pnode_b = unsafe { &mut *self.queue[higher].pnode };
        let join_left = pnode_b.priority - pnode.priority == 1;
        if join_left {
            self.queue[i].pnode = pnode_b as *mut _;
            let r = &mut pnode_b.bounds[1];
            *r = Self::index_add(len, *r, 1);
        }
        join_left
    }
}

impl Drop for RangePriorityQueue {
    fn drop(&mut self) {
        let mut last = ptr::null_mut();
        for i in 0..self.len {
            let i = Self::index_add(self.range(), self.head, i);
            let pnode = self.queue[i].pnode;
            if pnode != last {
                last = pnode;
                unsafe { Box::from_raw(pnode) };
            }
        }
    }
}

impl Extend<u32> for RangePriorityQueue {
    fn extend<T: IntoIterator<Item=u32>>(&mut self, iter: T) {
        for elem in iter {
            self.add(elem);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn add_out_of_bounds() {
        let mut q = RangePriorityQueue::new(4);
        q.add(4);
    }

    #[test]
    fn add_extremes() {
        let mut q = RangePriorityQueue::new(4);
        q.add(0);
        assert_eq!(Some(0), q.pop());
        q.add(3);
        assert_eq!(Some(3), q.pop());
    }

    #[test]
    fn add_combinations() {
        let mut q = RangePriorityQueue::new(4);
        q.add(0);
        q.add(1);
        q.add(2);
        q.add(3);
        assert_eq!(Some(0), q.pop());
        assert_eq!(Some(1), q.pop());
        assert_eq!(Some(2), q.pop());
        assert_eq!(Some(3), q.pop());
        q.add(3);
        q.add(2);
        q.add(1);
        q.add(0);
        assert_eq!(Some(3), q.pop());
        assert_eq!(Some(2), q.pop());
        assert_eq!(Some(1), q.pop());
        assert_eq!(Some(0), q.pop());
        q.add(0);
        q.add(1);
        q.add(2);
        q.add(2);
        assert_eq!(Some(2), q.pop());
        assert_eq!(Some(1), q.pop());
        assert_eq!(Some(0), q.pop());
        q.add(0);
        q.add(1);
        q.add(1);
        q.add(1);
        q.add(2);
        q.add(2);
        assert_eq!(Some(1), q.pop());
        assert_eq!(Some(2), q.pop());
        assert_eq!(Some(0), q.pop());
    }
}