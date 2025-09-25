pub trait UiEditRequestTrait {
    fn accept(&mut self) -> bool;
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub enum UiEditRequest<T> {
    #[default]
    None,

    Request(T),
    Editing(T),
}

impl<T: Copy> UiEditRequestTrait for UiEditRequest<T> {
    fn accept(&mut self) -> bool {
        match self {
            UiEditRequest::None | UiEditRequest::Editing(_) => false,
            UiEditRequest::Request(seq) => {
                *self = UiEditRequest::Editing(*seq);
                true
            }
        }
    }
}

impl<T> UiEditRequest<T> {
    pub fn option(self) -> Option<T> {
        match self {
            UiEditRequest::None => None,
            UiEditRequest::Request(seq) | UiEditRequest::Editing(seq) => Some(seq),
        }
    }
}
