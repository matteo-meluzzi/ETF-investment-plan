use std::rc::Rc;
use derive_new::new;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum RcList<T> {
    Node(T, Rc<RcList<T>>),
    Stop
}
impl<T> RcList<T> {
    pub fn iter(&self) -> RcListRefIterator<T> {
        RcListRefIterator::new(self)
    }
}

#[derive(new)]
pub struct RcListRefIterator<'a, T> {
    current_list: &'a RcList<T>
}
impl<'a, T> Iterator for RcListRefIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_list {
            RcList::Stop => None,
            RcList::Node(t, next) => {
                self.current_list = next.as_ref();
                Some(t)
            }
        }
    }
}
