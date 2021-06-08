use std::{collections::HashMap};
use dasp_graph::{NodeData, BoxedNodeSend, Processor, Buffer, Input, Node};
use petgraph::graph::{NodeIndex};
use petgraph::stable_graph::{StableDiGraph};
// use pest::Parser;
use pest::iterators::Pairs;
use glicol_parser::*;

pub mod oscillator; use oscillator::*;
use {sin_osc::*, saw_osc::SawOsc, squ_osc::SquOsc, tri_osc::TriOsc};

pub mod signal; use signal::*;
use {imp::*, const_sig::ConstSig, noise::Noise};

pub mod operation; use operation::*;
use {mul::Mul, add::Add};

pub mod filter; use filter::*;
use {lpf::*, hpf::*, apfgain::*, apfdecay::*, onepole::*,comb::*};

pub mod sampling; use sampling::*;
use {seq::*, sampler::*,speed::*, choose::*};

pub mod pass; use pass::*;

pub mod effect; use effect::*;
use {delayn::*, delay::*};

pub type GlicolNodeData = NodeData<BoxedNodeSend<128>, 128>;
pub type GlicolGraph = StableDiGraph<GlicolNodeData, (), u32>;
pub type GlicolProcessor = Processor<GlicolGraph, 128>;
pub type NodeResult = Result<(GlicolNodeData, Vec<String>), GlicolError>;

#[derive(Debug)]
pub enum GlicolError {
    NonExistControlNodeError(String),
    ParameterError((usize, usize)),
    SampleNotExistError((usize, usize)),
    InsufficientParameter((usize, usize)),
    NotModuableError((usize, usize)),
    ParaTypeError((usize, usize)),

}

// pub mod adc; // pub mod operator;
// pub mod envelope; // pub mod map; 
// pub mod phasor; // pub mod buf; 
// pub mod state; // pub mod pan; 
// pub mod delay; // pub mod reverb;
// use phasor::{Phasor}; // use envelope::EnvPerc;
// use map::{LinRange}; // use buf::{Buf};
// use state::{State}; // use pan::{Pan, Mix2};
// use reverb::{Plate};
pub fn make_node(
    name: &str,
    paras: &mut Pairs<Rule>,
    samples_dict: &HashMap<String, &'static[f32]>,
    sr: usize,
    bpm: f32,
) -> NodeResult {
    
    let alias = match name {
        "sp" => "sampler",
        "*" => "mul",
        "noiz" => "noise",
        _ => name
    };

    let modulable = match alias {
        "sin" => vec![Para::Modulable],
        "saw" => vec![Para::Modulable],
        "squ" => vec![Para::Modulable],
        "tri" => vec![Para::Modulable],
        "const" => vec![Para::Number(0.0)],
        "mul" => vec![Para::Modulable],
        "add" => vec![Para::Modulable],
        "lpf" => vec![Para::Modulable, Para::Number(1.0)],
        "hpf" => vec![Para::Modulable, Para::Number(1.0)],
        "sampler" => {
            // check potential errors
            if !samples_dict.contains_key(&paras.as_str().replace("\\", "")) {
                let p = paras.next().unwrap();
                let pos = (p.as_span().start(), p.as_span().end());
                return Err(GlicolError::SampleNotExistError(pos))
            }
            vec![]
        }, // bypass the process_parameters
        "seq" => vec![],
        "choose" => { vec![] },
        "delayn" => vec![Para::Number(1.0)],
        "delay" => vec![Para::Modulable],
        "onepole" => vec![Para::Modulable],
        "comb" => vec![Para::Number(10.), Para::Number(0.9), Para::Number(0.5), Para::Number(0.5)],
        "apfdecay" => vec![Para::Number(10.), Para::Number(0.8)],
        "apfgain" => vec![Para::Number(10.), Para::Number(0.5)],
        _ => vec![Para::Modulable], // pass
    };

    // this func checks if the parameters are correct
    let (p, mut refs) = process_parameters(paras, modulable)?;

    if name == "seq" {refs = process_seq(paras.as_str())?.2}
    
    let nodedata = match alias {
        "sin" => sin_osc!({freq: get_num(&p[0]), sr: sr}),
        "saw" => saw_osc!({freq: get_num(&p[0]), sr: sr}),
        "squ" => squ_osc!({freq: get_num(&p[0]), sr: sr}),
        "tri" => tri_osc!({freq: get_num(&p[0]), sr: sr}),
        "const" => const_sig!(get_num(&p[0])),
        "mul" => mul!(get_num(&p[0])),
        "add" => add!(get_num(&p[0])),
        "lpf" => rlpf!({cutoff: get_num(&p[0]), q: get_num(&p[1])}),
        "hpf" => rhpf!({cutoff: get_num(&p[0]), q: get_num(&p[1])}),

        "noise" => noise!(get_num(&p[0]) as u64),
        "imp" => imp!({freq: get_num(&p[0]), sr: sr}),
        "sampler" => {
            sampler!(samples_dict[&paras.as_str().replace("\\", "")])},
        "seq" => {
            let info = process_seq(paras.as_str()).unwrap();
            seq!({events: info.0, sidechain_lib: info.1, sr: sr, bpm: bpm})
        }
        "speed" => speed!(get_num(&p[0])),
        "choose" => choose!(get_notes(paras)?),
        "delayn" => delayn!(get_num(&p[0]) as usize),
        "delay" => delay!({delay: get_num(&p[0]), sr: sr}),
        "onepole" => onepole!(get_num(&p[0])),
        "comb" => comb!({delay: get_num(&p[0]), gain: get_num(&p[1]), feedforward: get_num(&p[2]), feedback: get_num(&p[3])}),
        "apfdecay" => apfdecay!({delay: get_num(&p[0]), decay: get_num(&p[1])}),
        "apfgain" => apfgain!({delay: get_num(&p[0]), gain: get_num(&p[1])}),
        _ => Pass::new()

        // "choose" => Choose::new(&mut paras)?,
        // "envperc" => EnvPerc::new(30.0, 50.0)?,
        // "pan" => Pan::new(&mut paras)?,
        // "buf" => Buf::new(&mut paras, 
        //     samples_dict)?,
        // "linrange" => LinRange::new(&mut paras)?,
        // "pha" => Phasor::new(&mut paras)?,
        // "state" => State::new(&mut paras)?,
        // "delay" => Delay::new(&mut paras)?,
        // "apf" => Allpass::new(&mut paras)?,
        // "comb" => Comb::new(&mut paras)?,
        // "mix" => Mix2::new(&mut paras)?,
        // "plate" => Plate::new(&mut paras)?,
        // "onepole" => OnePole::new(&mut paras)?,
        // "allpass" => AllpassGain::new(&mut paras)?,
        // "delayn" => DelayN::new(&mut paras)?,
        // "monosum" => MonoSum::new(&mut paras)?,
        // _ => Pass::new(name)?
    };
    Ok((nodedata, refs))
}


#[derive(Debug, Clone, PartialEq)]
/// Parameter of a node can be f32, String, or NodeIndex for sidechain
pub enum Para {
    Number(f32),
    Symbol(String),
    Index(NodeIndex),
    Modulable
}

fn get_num(p: &Para) -> f32 {
    match p {
        Para::Number(v) => *v,
        Para::Modulable => 0.0,
        _ => 0.0
    }
}

type Events = Vec::<(f64, String)>;
type Sidechain = HashMap::<String, usize>;

fn process_seq(pattern: &str) -> Result<(Events, Sidechain, Vec<String>), GlicolError> {
    let mut events = Vec::<(f64, String)>::new();
    let mut sidechain_count = 0;
    let mut sidechains = Vec::new();
    let mut sidechain_lib = Sidechain::new();
    let split: Vec<&str> = pattern.split(" ").collect();
    let len_by_space = split.len();
    let compound_unit = 1.0 / len_by_space as f64;

    for (i, compound) in split.iter().enumerate() {
        let c = compound.replace("_", "$_$");
        let notes = c.split("$").filter(|x|x!=&"").collect::<Vec<_>>();

        let notes_len = notes.len();
        for (j, x) in notes.iter().enumerate() {
            let relative_time = i as f64 / len_by_space as f64 
            + (j as f64/ notes_len as f64 ) * compound_unit;

            if x.contains("~") {
                sidechains.push(x.to_string());
                sidechain_lib.insert(x.to_string(), sidechain_count);
                sidechain_count += 1;
            }

            if x != &"_" {
                events.push((relative_time, x.to_string()))
            }
        }
    }
    Ok((events, sidechain_lib, sidechains))
}

fn get_notes(paras: &mut Pairs<Rule>) -> Result<Vec::<f32>, GlicolError> {
    let split: Vec<&str> = paras.as_str().split(" ").collect();
    let mut note_list = Vec::<f32>::new();
    println!("split{:?}", split);
    for note in split {
        match note.parse::<f32>() {
            Ok(v) => note_list.push(v),
            Err(_) => {
                let p = paras.next().unwrap();
                let pos = (p.as_span().start(), p.as_span().end());
                return Err(GlicolError::ParameterError(pos))
            }
        }
    }
    println!("note_list{:?}", note_list);
    Ok(note_list)
}

#[macro_export]
macro_rules! mono_node {
    ($body:expr) => {
        NodeData::new1( BoxedNodeSend::new(($body)))
    };
}

#[macro_export]
macro_rules! ndef {
    ($struct_name: ident, $channel_num: ident, {$code_str: expr}) => {
        pub struct $struct_name {
            engine: Engine
        }
        
        impl $struct_name {
            pub fn new(paras: &mut Pairs<Rule>) -> Result<
            (NodeData<BoxedNodeSend<128>, 128>, Vec<String>), EngineError> {
                let mut engine = Engine::new();
                engine.set_code(&format!($code_str, a=paras.as_str()));
                engine.make_graph()?;
                Ok((NodeData::$channel_num(BoxedNodeSend::new( Self {
                    engine
                })), vec![]))
            }
        }
        
        impl Node<128> for $struct_name {
            fn process(&mut self, inputs: &[Input<128>], output: &mut [Buffer<128>]) {
                // self.engine.input(inputs); // mono or stereo?
                let mut input = inputs[0].buffers()[0].clone();
                let buf = self.engine.gen_next_buf_128(&mut input).unwrap();
                match output.len() {
                    1 => {
                        for i in 0..128 {
                            output[0][i] = buf.0[i];
                        }
                    },
                    2 => {
                        for i in 0..128 {
                            output[0][i] = buf.0[i];
                            output[1][i] = buf.0[i+128];
                        }
                    },
                    _ => {}
                }
            }
        }
    };
}

// TODO: make the all in 3 macros

#[macro_export]
macro_rules! const_sig {
    ($data: expr) => {
        ConstSig::new($data)
    };
}

#[macro_export]
macro_rules! imp {
    ({$($para: ident: $data:expr),*}) => {
         (
            Impulse::new()$(.$para($data))*.build()
        )
    }
}

#[macro_export]
macro_rules! noise {
    () => { // controlled by modulator, no need for value
        Noise::new(42)
    };

    ($data: expr) => {
        Noise::new($data)
    };
}

#[macro_export]
macro_rules! speed {
    ($data: expr) => {
        Speed::new($data)
    };
}

#[macro_export]
macro_rules! mul {
    () => { // controlled by modulator, no need for value
        Mul::new(0.0)
    };

    ($data: expr) => {
        Mul::new($data)
    };
}

#[macro_export]
macro_rules! add {
    () => {
        Add::new(0.0)
    };

    ($data: expr) => {
        Add::new($data)
    };
}


#[macro_export]
macro_rules! sin_osc {
    ({$($para: ident: $data:expr),*}) => {
         (
            SinOsc::new()$(.$para($data))*.build()
        )
    }
}

#[macro_export]
macro_rules! tri_osc {
    ({$($para: ident: $data:expr),*}) => {
         (
            TriOsc::new()$(.$para($data))*.build()
        )
    }
}

#[macro_export]
macro_rules! squ_osc {
    ({$($para: ident: $data:expr),*}) => {
         (
            SquOsc::new()$(.$para($data))*.build()
        )
    }
}

#[macro_export]
macro_rules! saw_osc {
    ({$($para: ident: $data:expr),*}) => {
         (
            SawOsc::new()$(.$para($data))*.build()
        )
    }
}