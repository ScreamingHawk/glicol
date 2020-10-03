use dasp_graph::{Buffer, Input, Node};
use pest::iterators::Pairs;
use super::super::{HashMap, Rule, NodeData, BoxedNodeSend};

pub struct Sequencer {
    events: Vec<(f64, String)>,
    speed: f32,
    pub step: usize,
    sidechain_lib: HashMap<String, usize>
}

impl Sequencer {
    pub fn new(paras: &mut Pairs<Rule>) -> (NodeData<BoxedNodeSend>, Vec<String>) {
        let mut events = Vec::<(f64, String)>::new();

        let mut sidechains = Vec::<String>::new();
        let mut sidechain_id = 0;
        let mut sidechain_lib = HashMap::<String, usize>::new();
        
        // para -> seq -> compound -> note -> midi/ref/rest
        // println!("paras input to seq", paras.as_str());
        // let mut paras = paras.next().unwrap().into_inner();
        // .next().unwrap()
        // let seq = paras.next().unwrap();

        let seq = paras;
        // .into_inner().into_inner()
        
        let mut compound_index = 0;
        let seq_by_space: Vec<pest::iterators::Pair<Rule>> = 
        seq.clone().collect();

        for compound in seq {
            let mut shift = 0;
            // calculate the length of seq
            let compound_vec: Vec<pest::iterators::Pair<Rule>> = 
            compound.clone().into_inner().collect();

            for note in compound.into_inner() {
                if !note.as_str().parse::<i32>().is_ok() & (note.as_str() != "_") {
                    sidechains.push(note.as_str().to_string());
                    sidechain_lib.insert(note.as_str().to_string(), sidechain_id);
                    sidechain_id += 1;
                }
            
                let seq_shift = 1.0 / seq_by_space.len() as f64 * 
                compound_index as f64;
                
                let note_shift = 1.0 / compound_vec.len() as f64 *
                shift as f64 / seq_by_space.len() as f64;

                // relative_pitch can be a ref
                if note.as_str() != "_" {
                    let relative_pitch = note.as_str().to_string();
                    let relative_time = seq_shift + note_shift;
                    events.push((relative_time, relative_pitch));
                }
                shift += 1;
                // }
            }
            compound_index += 1;
        }

        // println!("events {:?}", events);

        (NodeData::new1(BoxedNodeSend::new( Self {
            events: events,
            speed: 1.0,
            step: 0,
            sidechain_lib: sidechain_lib
        })), sidechains)
    }
}

impl Node for Sequencer {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer]) {
        
        let mut has_speed_input = false;
        
        if inputs.len() > 0 {
            // speed input is set as [ f32, 0.0, 0.0 ... ], so it's identical
            // NOTE! inputs are in reverse order

            // println!("input0 {} {}", inputs[0].buffers()[0][0],inputs[0].buffers()[0][1]);
            // println!("input0 {}, input1 {}, input2 {}", inputs[0].buffers()[0][0], 
            // inputs[1].buffers()[0][0], inputs[2].buffers()[0][0]);
            // println!("input0 {}, input1 {}, input2 {}", inputs[0].buffers()[0][1], 
            // inputs[1].buffers()[0][1], inputs[2].buffers()[0][1]);
            let last = inputs.len() - 1;
            if (inputs[last].buffers()[0][0] > 0.0) & (inputs[last].buffers()[0][1] == 0.0) {
                self.speed = inputs[last].buffers()[0][0];
                has_speed_input = true;
            }
        }

        // println!("speed {}", self.speed);
        // let relative_time = event.0;
        // let relative_pitch = event.1; a ratio for midi 60 freq
        let bar_length = 88200.0 / self.speed as f64;
        for i in 0..64 {
            output[0][i] = 0.0;

            for event in &self.events {
                if (self.step % (bar_length as usize)) == ((event.0 * bar_length) as usize) {

                    output[0][i] = match event.1.parse::<f32>() {
                        Ok(val) => val,
                        Err(_why) => {
                            let len = inputs.len();

                            // there are cases:
                            // - no speed input, but has several sidechains
                            // - one speed input, no sidechain,
                            // - one speed input. several sidechains

                            let index = len - 1 - 
                            self.sidechain_lib[&event.1] - has_speed_input as usize;
                            // println!("index {}", index);
                            inputs[index].buffers()[0][i]
                        }
                    };
                    output[0][i] = 2.0f32.powf((output[0][i] - 60.0)/12.0);
                }
            }
            self.step += 1;
        }
    }
}

pub struct Speed {
    pub speed: f32,
    has_mod: bool
}

impl Speed {
    pub fn new(paras: &mut Pairs<Rule>) -> (NodeData<BoxedNodeSend>, Vec<String>) {

        let speed: String = paras.next().unwrap().as_str().to_string()
        .chars().filter(|c| !c.is_whitespace()).collect();

        let is_float = speed.parse::<f32>();

        if is_float.is_ok() {
            (NodeData::new1(BoxedNodeSend::new(
                Self {speed: is_float.unwrap(), has_mod: false})),
            vec![])
        } else {
            (NodeData::new1(BoxedNodeSend::new(
                Self {speed: 1.0, has_mod: true})),
            vec![speed])
        }
    }
}
impl Node for Speed {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer]) {

        if self.has_mod {
            assert!(inputs.len() > 0);
            let mod_buf = &mut inputs[0].buffers();
            // let mod_buf = &mut inputs[1].buffers();
            // for i in 0..64 {
            output[0][0] = mod_buf[0][0];
            // }
        } else {
            assert_eq!(inputs.len(), 0);
            // output[0] = inputs[0].buffers()[0].clone();
            output[0][0] = self.speed as f32;
            // output[0].iter_mut().for_each(|s| *s = self.speed as f32);
        }
        // if inputs.len() > 0 {
    }
}

// impl Node for Speed {
//     fn process(&mut self, _inputs: &[Input], output: &mut [Buffer]) {
//         for o in output {
//             o.iter_mut().for_each(|s| *s = self.speed);
//         }
//     }
// }