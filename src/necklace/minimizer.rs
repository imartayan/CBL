use core::cmp::min;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct LexMinQueue<const WIDTH: usize, T: Ord + Copy> {
    deq: VecDeque<(T, usize)>,
    min_pos: VecDeque<usize>,
    pos: usize,
}

impl<const WIDTH: usize, T: Ord + Copy> LexMinQueue<WIDTH, T> {
    pub fn new() -> Self {
        Self {
            deq: VecDeque::with_capacity(WIDTH),
            min_pos: VecDeque::with_capacity(WIDTH),
            pos: 0,
        }
    }

    pub fn iter_min_pos(&self) -> impl Iterator<Item = usize> + '_ {
        self.min_pos
            .iter()
            .map(|&pos| (pos + WIDTH - self.pos) % WIDTH)
    }

    pub fn insert_full<I: DoubleEndedIterator<Item = T>>(&mut self, vals: I) {
        self.deq.clear();
        self.min_pos.clear();
        let mut vals_rev = vals.rev();
        let mut minimizer = vals_rev.next().unwrap();
        let mut pos = (self.pos + WIDTH - 1) % WIDTH;
        self.deq.push_front((minimizer, pos));
        for u in vals_rev.take(WIDTH - 1) {
            pos = (pos + WIDTH - 1) % WIDTH;
            if u <= minimizer {
                minimizer = u;
                self.deq.push_front((minimizer, pos));
            }
        }
        while self.min_pos.len() < self.deq.len() && self.deq[self.min_pos.len()].0 == self.deq[0].0
        {
            self.min_pos.push_back(self.deq[self.min_pos.len()].1);
        }
    }

    pub fn insert(&mut self, u: T) {
        if !self.deq.is_empty() && self.deq[0].1 == self.pos {
            self.deq.pop_front();
            self.min_pos.pop_front();
        }
        let mut i = self.deq.len();
        while i > 0 && self.deq[i - 1].0 > u {
            i -= 1;
        }
        self.deq.truncate(i);
        self.min_pos.truncate(i);
        self.deq.push_back((u, self.pos));
        while self.min_pos.len() < self.deq.len() && self.deq[self.min_pos.len()].0 == self.deq[0].0
        {
            self.min_pos.push_back(self.deq[self.min_pos.len()].1);
        }
        self.pos = (self.pos + 1) % WIDTH;
    }

    pub fn insert2(&mut self, u: T, v: T) {
        let next_pos = (self.pos + 1) % WIDTH;
        if !self.deq.is_empty() && self.deq[0].1 == self.pos {
            self.deq.pop_front();
            self.min_pos.pop_front();
        }
        if !self.deq.is_empty() && self.deq[0].1 == next_pos {
            self.deq.pop_front();
            self.min_pos.pop_front();
        }
        let w = min(u, v);
        let mut i = self.deq.len();
        while i > 0 && self.deq[i - 1].0 > w {
            i -= 1;
        }
        self.deq.truncate(i);
        self.min_pos.truncate(i);
        if u <= v {
            self.deq.push_back((u, self.pos));
        }
        self.deq.push_back((v, next_pos));
        while self.min_pos.len() < self.deq.len() && self.deq[self.min_pos.len()].0 == self.deq[0].0
        {
            self.min_pos.push_back(self.deq[self.min_pos.len()].1);
        }
        self.pos = (next_pos + 1) % WIDTH;
    }
}

impl<const WIDTH: usize, T: Ord + Copy> Default for LexMinQueue<WIDTH, T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    const BITS: usize = 8;
    const M: usize = 5;
    const WIDTH: usize = BITS - M + 1;

    #[test]
    fn test_lex_min_queue_insert_full() {
        let mut min_queue = LexMinQueue::<WIDTH, _>::new();
        min_queue.insert_full([2, 1, 2, 1].iter());
        assert_eq!(
            min_queue.iter_min_pos().collect_vec(),
            vec![WIDTH - 3, WIDTH - 1],
            "{:?}",
            min_queue
        );
    }

    #[test]
    fn test_lex_min_queue_insert() {
        let mut min_queue = LexMinQueue::<WIDTH, _>::new();
        min_queue.insert(3);
        assert_eq!(
            min_queue.iter_min_pos().collect_vec(),
            vec![WIDTH - 1],
            "{:?}",
            min_queue
        );
        min_queue.insert(1);
        assert_eq!(
            min_queue.iter_min_pos().collect_vec(),
            vec![WIDTH - 1],
            "{:?}",
            min_queue
        );
        min_queue.insert(2);
        assert_eq!(
            min_queue.iter_min_pos().collect_vec(),
            vec![WIDTH - 2],
            "{:?}",
            min_queue
        );
        min_queue.insert(3);
        assert_eq!(
            min_queue.iter_min_pos().collect_vec(),
            vec![WIDTH - 3],
            "{:?}",
            min_queue
        );
        min_queue.insert(1);
        assert_eq!(
            min_queue.iter_min_pos().collect_vec(),
            vec![WIDTH - 4, WIDTH - 1],
            "{:?}",
            min_queue
        );
        min_queue.insert(2);
        assert_eq!(
            min_queue.iter_min_pos().collect_vec(),
            vec![WIDTH - 2],
            "{:?}",
            min_queue
        );
    }
}
