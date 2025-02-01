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
    pub fn from_previous_savestate(previous_location: &TokenQueueSlice) -> TokenQueueSlice {
        TokenQueueSlice{
            index: previous_location.index,
            max_index: previous_location.max_index
        }
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
        self.max_index - self.index
    }

    pub fn next(&mut self){
        self.index += 1;
    }
}