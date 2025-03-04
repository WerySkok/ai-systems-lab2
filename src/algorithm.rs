#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
pub enum Optimum {
    Minimum,
    Maximum,
}

#[derive(Clone, Debug)]
pub struct Agent {
    pub x: f64,
    pub y: Option<f64>,
}

impl Agent {
    pub fn new(x: f64) -> Self {
        Agent { x, y: None }
    }

    pub fn mutate(&mut self, strangeness: f64, chance: f64) {
        if rand::random::<f64>() <= chance {
            if rand::random::<f64>() > 0.5 {
                self.x += strangeness;
            } else {
                self.x -= strangeness;
            }
        }
    }

    pub fn calculate<F>(&mut self, fun: F)
    where
        F: Fn(f64) -> f64,
    {
        self.y = Some(fun(self.x))
    }
}

#[derive(Clone, Debug)]
pub struct GenerationData {
    pub survivors: Vec<Agent>,
    pub discarded: Vec<Agent>,
}
