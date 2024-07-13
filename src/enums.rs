#[derive(Default,Clone,PartialEq, Eq)]
    pub enum StreamingState{
        START,
        PAUSE,
        BLANK,
        #[default]
        STOP,
    }