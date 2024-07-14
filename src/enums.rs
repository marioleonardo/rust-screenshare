#[derive(Default,Debug,Clone,PartialEq, Eq)]
    pub enum StreamingState{
        START,
        PAUSE,
        BLANK,
        #[default]
        STOP,
    }