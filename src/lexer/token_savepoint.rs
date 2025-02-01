
/**
 * specifies a slice of the token list
 */
pub struct TokenQueueSlice {
    index: usize,
    max_index: usize//where the end of the list is (so that you can slice)
}

impl TokenQueueSlice {
    pub fn new() -> TokenQueueSlice{
        TokenQueueSlice{
            index:0,
            max_index: usize::max_value()
        }
    }
    /**
     * construct a slice from the start and end indexes
     * max_index is usually usize max
     */
    pub fn new_from_bounds(index: usize, max_index: usize) -> TokenQueueSlice {
        TokenQueueSlice {
            index,
            max_index
        }
    }

    pub fn from_previous_savestate(previous_location: &TokenQueueSlice) -> TokenQueueSlice {
        TokenQueueSlice{
            index: previous_location.index,
            max_index: previous_location.max_index
        }
    }

    /**
     * converts self.index to be the same as to_copy.get_index()
     */
    pub fn copy_start_index(&mut self, to_copy: &TokenQueueSlice) {
        self.index = to_copy.get_index();
    }

    pub fn get_index(&self) -> usize{
        self.index
    }
    /**
     * get the index past the last item in the sliece
     */
    pub fn get_slice_max_idx(&self) -> usize {
        self.max_index
    }

    /**
     * returns the length of this slice
     */
    pub fn get_slice_size(&self) -> usize {
        if self.index > self.max_index {
            return 0;
        }
        self.max_index - self.index
    }

    /**
     * returns a copy of self with an incremented index
     */
    pub fn next_clone(&self) -> Self {
        TokenQueueSlice { index: self.get_index()+1, max_index: self.get_slice_max_idx() }
    }

    pub fn next(&mut self){
        self.index += 1;
    }
}