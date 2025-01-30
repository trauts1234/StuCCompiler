pub struct TokenQueueLocation {
    index: usize
}

impl TokenQueueLocation {
    pub fn new() -> TokenQueueLocation{
        TokenQueueLocation{
            index:0
        }
    }
    pub fn from_previous_savestate(previous_location: &TokenQueueLocation) -> TokenQueueLocation {
        TokenQueueLocation{
            index: previous_location.index
        }
    }

    pub fn get_index(&self) -> usize{
        self.index
    }
    pub fn next(&mut self){
        self.index += 1;
    }
}