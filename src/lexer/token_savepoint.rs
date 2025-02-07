
/**
 * specifies a slice of the token list
 */
#[derive(Clone)]
pub struct TokenQueueSlice {
    pub(crate) index: usize,
    pub(crate) max_index: usize//one past end of slice
}

impl TokenQueueSlice {
    pub fn new() -> TokenQueueSlice{
        TokenQueueSlice{
            index:0,
            max_index: usize::max_value()
        }
    }

    pub fn from_previous_savestate(previous_location: &TokenQueueSlice) -> TokenQueueSlice {
        TokenQueueSlice{
            index: previous_location.index,
            max_index: previous_location.max_index
        }
    }

    /**
     * returns the length of this slice
     */
    pub fn get_slice_size(&self) -> usize {
        if self.index > self.max_index {
            panic!("backwards slice detected!");
        }
        self.max_index - self.index//max_index is one past the end
    }

    /**
     * returns a copy of self with an incremented index
     */
    pub fn next_clone(&self) -> Self {
        TokenQueueSlice { index: self.index+1, max_index: self.max_index }
    }

    pub fn next(&mut self){
        self.index += 1;
    }
}