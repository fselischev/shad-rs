#![forbid(unsafe_code)]

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundOutcome {
    BothCooperated,
    LeftCheated,
    RightCheated,
    BothCheated,
}

pub struct Game {
    left: Box<dyn Agent>,
    right: Box<dyn Agent>,
}

impl Game {
    pub fn new(left: Box<dyn Agent>, right: Box<dyn Agent>) -> Self {
        Self { left, right }
    }

    pub fn left_score(&self) -> i32 {
        self.left.score()
    }

    pub fn right_score(&self) -> i32 {
        self.right.score()
    }

    pub fn play_round(&mut self) -> RoundOutcome {
        let left_action = self.left.action(self.right.last_play());
        let right_action = self.right.action(self.left.last_play());

        match left_action {
            Play::Cheat => match right_action {
                Play::Cheat => RoundOutcome::BothCheated,
                Play::Cooperate => {
                    self.left.upd_score(3);
                    self.right.upd_score(-1);
                    RoundOutcome::LeftCheated
                }
            },
            Play::Cooperate => match right_action {
                Play::Cheat => {
                    self.left.upd_score(-1);
                    self.right.upd_score(3);
                    RoundOutcome::RightCheated
                }
                Play::Cooperate => {
                    self.left.upd_score(2);
                    self.right.upd_score(2);
                    RoundOutcome::BothCooperated
                }
            },
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
pub trait Agent: Action + Score {}

pub trait Action {
    fn last_play(&self) -> Play;
    fn action(&mut self, last_play: Play) -> Play;
}

pub trait Score {
    fn score(&self) -> i32;
    fn upd_score(&mut self, value: i32);
}

// just playing around with declarative macros to get rid off the boilerplate
macro_rules! impl_score {
    ($agent:ident) => {
        impl Score for $agent {
            fn score(&self) -> i32 {
                self.score
            }

            fn upd_score(&mut self, value: i32) {
                self.score += value;
            }
        }
    };
}

#[derive(Default)]
pub struct CheatingAgent {
    score: i32,
}

impl CheatingAgent {
    pub fn new() -> Self {
        Self::default()
    }
}

impl_score!(CheatingAgent);
impl Agent for CheatingAgent {}
impl Action for CheatingAgent {
    fn last_play(&self) -> Play {
        Play::Cheat
    }

    fn action(&mut self, _: Play) -> Play {
        Play::Cheat
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct CooperatingAgent {
    score: i32,
}

impl CooperatingAgent {
    pub fn new() -> Self {
        Self::default()
    }
}

impl_score!(CooperatingAgent);
impl Agent for CooperatingAgent {}
impl Action for CooperatingAgent {
    fn last_play(&self) -> Play {
        Play::Cooperate
    }

    fn action(&mut self, _: Play) -> Play {
        Play::Cooperate
    }
}

////////////////////////////////////////////////////////////////////////////////

// always cooperates until first betrayal, then always cheats
#[derive(Default)]
pub struct GrudgerAgent {
    score: i32,
    not_first_play: bool,
    cheated_once: Play,
}

impl GrudgerAgent {
    pub fn new() -> Self {
        Self::default()
    }
}

impl_score!(GrudgerAgent);
impl Agent for GrudgerAgent {}
impl Action for GrudgerAgent {
    fn last_play(&self) -> Play {
        Play::Cooperate
    }

    fn action(&mut self, last_play: Play) -> Play {
        if !self.not_first_play {
            self.not_first_play = true;
            return Play::Cooperate;
        }
        if let Play::Cheat = last_play {
            self.cheated_once = Play::Cheat;
        }

        self.cheated_once
    }
}

////////////////////////////////////////////////////////////////////////////////

// cooperates first, then repeats the last turn of opponent
#[derive(Default)]
pub struct CopycatAgent {
    score: i32,
    not_first_play: bool,
    last_play: Play,
}

impl CopycatAgent {
    pub fn new() -> Self {
        Self::default()
    }
}

impl_score!(CopycatAgent);
impl Agent for CopycatAgent {}
impl Action for CopycatAgent {
    fn last_play(&self) -> Play {
        self.last_play
    }

    fn action(&mut self, last_play: Play) -> Play {
        if !self.not_first_play {
            self.not_first_play = true;
            return Play::Cooperate;
        }

        self.last_play = last_play;
        last_play
    }
}

////////////////////////////////////////////////////////////////////////////////

// begins with sequence "cooperate", "cheat", "cooperate", "cooperate". If opponent never cheated, then always cheats. Otherwise, plays as copycat agent
#[derive(Default)]
pub struct DetectiveAgent {
    score: i32,
    counter: u32,
    last_action: Play,
    cheated: bool,
}

impl DetectiveAgent {
    pub fn new() -> Self {
        Self::default()
    }
}

impl_score!(DetectiveAgent);
impl Agent for DetectiveAgent {}
impl Action for DetectiveAgent {
    fn last_play(&self) -> Play {
        self.last_action
    }

    fn action(&mut self, last_action: Play) -> Play {
        if let Play::Cheat = last_action {
            self.cheated = true;
        }

        while self.counter < 5 {
            self.counter += 1;
            match self.counter {
                1 | 3 | 4 => {
                    self.last_action = Play::Cooperate;
                    return self.last_action;
                }
                2 => {
                    self.last_action = Play::Cheat;
                    return self.last_action;
                }
                _ => {}
            }
        }

        if self.cheated {
            self.last_action = last_action;
            last_action
        } else {
            self.last_action = Play::Cheat;
            self.last_action
        }
    }
}

///////////////////////////////

#[derive(Copy, Clone, Default)]
pub enum Play {
    Cheat,
    #[default]
    Cooperate,
}
