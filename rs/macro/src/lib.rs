use proc_macro::{TokenStream, TokenTree};
use quote::quote;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};

#[proc_macro]
pub fn def_node(input: TokenStream) -> TokenStream {
    let mut code: String = "".to_owned();
    // let mut name = Ident::new("A", Span::call_site());
    // let mut macroname = Ident::new("a", Span::call_site());
    // let mut paras = TokenStream2::new();
    // let mut varname = vec![];
    // let mut variable = vec![];
    // let mut behavior = TokenStream2::new();
    
    let mut input_iter = input.into_iter();
    let object_all = input_iter.next().unwrap();
    let object_all_stream = match object_all {
        TokenTree::Group(g) => TokenStream2::from(g.stream()),
        _ => unimplemented!()
    };

    let mut i = object_all_stream.into_iter();
    let mut f = i.next();
    while f.is_some() {
        let raw = f.clone().unwrap();
        let item = f.unwrap().to_string();
        println!("item {}", &item);
        // let raw = f.clone().unwrap();
        // let item = f.unwrap().to_string();
        // if item.contains("{") & !item.contains(":") {
        //     println!("raw {:?}", raw); // raw is tokentree
        //     // let procmacrots = TokenStream::from(raw.clone());
        //     behavior = match raw {
        //         TokenTree::Group(g) => TokenStream2::from(g.stream()),
        //         _ => unimplemented!()
        //     };
        // } else if item.contains("{") & item.contains(":") { // this block is glicol syntax
        //     let glicol_code = match raw {
        //         TokenTree::Group(g) => TokenStream2::from(g.stream()),
        //         _ => unimplemented!()
        //     };
        //     // println!("glicol item is {:?}", glicol_code.clone());
        //     let mut glicol_code_iter = glicol_code.into_iter();
        //     let mut element = glicol_code_iter.next();
        //     // to calculate the args
        //     // to get the output chains into reference and added
        //     while element.is_some() {
        //         let item = element.unwrap().to_string();
        //         if &item == "#" {
        //             element = glicol_code_iter.next();
        //             let var = element.clone().unwrap().to_string();
                    
        //             code.push_str("{");
        //             code.push_str(&var);
        //             code.push_str("}");
        //             code.push_str(" ");
        //             if !varname.contains(&var) {
        //                 varname.push(var.clone());
        //                 variable.push(Ident::new(&element.unwrap().to_string(), Span::call_site()));
        //             }
        //         }
        //         element = glicol_code_iter.next();
        //     }
        // } else {

        // }
        f = i.next();
    }
    let o = quote!(
        pub fn preprocessor(chain_name:&str, node_name: &str, paras: &mut Pairs<Rule>, source: Vec<String>) -> Result<(String, String, Vec<String>), GlicolError> {
            
            let mut inplace_code = String::new();
            let mut appendix_code = String::new();
            let mut to_sink = vec![];
        
            let (target_paras, mut inplace, mut appendix) = match node_name {
                // "mul" => (vec![1.0], "mul ~mulmod_CHAIN_NAME", "~mulmod_CHAIN_NAME: const_sig PARA_0"),
                "bd" => (vec![0.3], "sin ~pitchCHAIN_NAME >> mul ~envbCHAIN_NAME >> mul 0.8", "~envbCHAIN_NAME: ~triggerbCHAIN_NAME >> envperc 0.01 PARA_0;~env_pitchCHAIN_NAME: ~triggerbCHAIN_NAME >> envperc 0.01 0.1;~pitchCHAIN_NAME: ~env_pitchCHAIN_NAME >> mul 50 >> add 60;~triggerbCHAIN_NAME: SOURCE;"),
                _ => {
                    inplace_code = node_name.to_owned();
                    inplace_code.push_str(" ");
                    inplace_code.push_str(paras.as_str());
                    println!("inplace_code {}", inplace_code);
                    to_sink = source;
                    to_sink.push(inplace_code.clone());
                    return Ok((inplace_code, appendix_code, to_sink));
                    (vec![], "", "")
                }
            };
        
            inplace_code = inplace.replace("CHAIN_NAME", &chain_name);
            appendix_code = appendix.replace("CHAIN_NAME", &chain_name);
            appendix_code = appendix_code.replace("SOURCE", &source.join(" >> "));
        
            for i in 0..target_paras.len() {
                match paras.next() {
                    Some(v) => {
                        let p = process_para(target_paras[i], v.as_str())?;
                        let current_para = format!("PARA_{}", i);
                        inplace_code = inplace_code.replace(&current_para, &format!("{}", p) );
                        appendix_code = appendix_code.replace(&current_para, &format!("{}", p) );
                    }
                    None => { return Err(GlicolError::InsufficientParameter((0,0))) }
                }
            }
            // panic!(appendix_code);
        
            match node_name {
                "bd" => {
                    to_sink = inplace_code.split(">>").map(|a|a.to_owned()).collect()
                },
                _ => {}
            }
        
            Ok( (inplace_code, appendix_code, to_sink) )
        }
        
        fn process_para(default: f32, input: &str) -> Result<String, GlicolError> {
            if input == "_" {
                return Ok(format!("{}", default))
            } else if input.parse::<f32>().is_ok() {
                return Ok(input.to_owned())
            } else {
                return Ok(input.to_owned())
            }
        }
        
    );
    o.into()
}


/// This is just a proof of concept
#[proc_macro]
pub fn make_node(input: TokenStream) -> TokenStream {
    // let code = &input.to_string();
    // let mut N: usize = 64;
    let mut code: String = "".to_owned();
    let mut name = Ident::new("A", Span::call_site());
    let mut macroname = Ident::new("a", Span::call_site());
    let mut varname = vec![];
    let mut variable = vec![];
    // let mut paras = TokenStream2::new();
    let mut behavior = TokenStream2::new();
    
    let mut i = input.into_iter();
    let mut f = i.next();
    while f.is_some() {
        let raw = f.clone().unwrap();
        let item = f.unwrap().to_string();
        // println!("{}", &item);
        if &item == "#" {
            f = i.next();
            let var = f.clone().unwrap().to_string();
            
            code.push_str("{");
            code.push_str(&var);
            code.push_str("}");
            code.push_str(" ");
            if !varname.contains(&var) {
                varname.push(var.clone());
                variable.push(Ident::new(&f.unwrap().to_string(), Span::call_site()));
            }
        } else if &item == "@" {
            f = i.next();
            // println!("hi {}", &f.unwrap());
            let namestr = &f.unwrap().to_string();
            name = Ident::new(namestr, Span::call_site());
            macroname = Ident::new(&namestr.to_lowercase(), Span::call_site());

        // } else if item.contains("(") {
        //     // println!("raw {:?}", raw); // raw is tokentree
        //     let procmacrots = TokenStream::from(raw.clone());
        //     paras = TokenStream2::from(procmacrots);
        //     // paras = item.replace(&['(', ')'][..], "");
        } else if item.contains("{") {
            // println!("raw {:?}", raw); // raw is tokentree
            // let procmacrots = TokenStream::from(raw.clone());
            behavior = match raw {
                TokenTree::Group(g) => TokenStream2::from(g.stream()),
                _ => unimplemented!()
            }
        } else if &item == ">" {
            code.push_str(&item);
            code.push_str(&item);
            i.next();
        } else if &item == "~" {
            code.push_str(&item);
            f = i.next();
            code.push_str(&f.unwrap().to_string());
            code.push_str(" ");
        } else if &item == "-" {
            code.push_str(&item);
            f = i.next();
            code.push_str(&f.unwrap().to_string());
            // i.next();
        } else {
            code.push_str(&item);
            code.push_str(" ");
        }
        f = i.next();
    }
    // println!("code: {} \n\nnodename: {:?}  \n\nvariable {:?}  \n\nparas {:?} \n\nbehavior {:?}",code, name, variable, paras, behavior);
    let o = quote!(

        pub struct #name<const N: usize> {
            graph: SimpleGraph<N>
        }
        
        impl<const N: usize> #name<N> {
            pub fn new(args: Vec<f32>) -> GlicolNodeData<N> {
                #behavior
                let graph = SimpleGraph::<N>::new(format!(#code, #(#variable = #variable),*).as_str());
                mono_node!( N, Self { graph } )
            }
        }

        impl<const N: usize> Node<N> for #name<N> {
            fn process(&mut self, inputs: &[Input<N>], output: &mut [Buffer<N>]) {       
                let mut input = [0.0; N];
                for i in 0..N {
                    input[i] = inputs[0].buffers()[0][i];
                }
                // println!("inputs {:?}", input);
                let out = self.graph.next_block(&mut input);
                for i in 0..N {
                    output[0][i] = out[i];
                    // output[1][i] = out[i+N];
                }
                // println!("out {:?}", out);
            }
        }
        
        // this is just ok for one-parameter node
        #[macro_export]
        macro_rules! #macroname{
            ($size: expr => $para:expr) => {
                #name::<$size>::new($para)
            };
        }
    );
    o.into()
}

#[proc_macro]
pub fn register_extensions(input: TokenStream) -> TokenStream {
    // println!("register input {:?}", input);
    let mut stream = input.into_iter();
    let mut token = stream.next();
    // let mut lib = HashMap::<String, u8>::new();
    let mut key_up = vec![];
    let mut key_low_str = vec![];
    let mut key_low = vec![];
    let mut para_num = vec![];
    while token.is_some() {
        match token.unwrap() {
            TokenTree::Group(_) => {},
            TokenTree::Ident(item) => { 
                key_up.push( item.clone() ); 
                key_low_str.push(item.clone().to_string().to_lowercase());
                key_low.push( Ident::new(&item.to_string().to_lowercase(), Span::call_site()) );
                
            },
            TokenTree::Punct(_) => { },
            TokenTree::Literal(item) => {
                let n = item.clone().to_string().parse::<u8>().unwrap();
                para_num.push(n);
            },
        }
        token = stream.next();
    };
    let o = quote!(
        pub mod nodes;
        use nodes::*;

        pub fn make_node_ext<const N: usize>(
            name: &str,
            paras: &mut Pairs<Rule>,
            pos: (usize, usize),
            samples_dict: &HashMap<String, &'static[f32]>,
            sr: usize,
            bpm: f32,
        ) -> Option<GlicolNodeData<N>> {
            let n = match name {
                #( #key_low_str => #para_num,  )*
                _ => return None
            };

            // if paras.as_str() == "_" {
            //     let node = match name {
            //         #( #key_low_str => #key_low!( N => args ), )*
            //         _ => unimplemented!()
            //     };
            // }

            let mut args: Vec<f32> = paras.as_str().split(" ").filter(|c| c != &"").map(|x|x.parse::<f32>().unwrap()).collect();
            // println!("args {:?}", args);
            // assert_eq!(args.len(), n as usize);
            let node = match name {
                #( #key_low_str => #key_low!( N => args ), )*
                _ => unimplemented!()
            };
            
            Some(node)
        }
    );
    // println!("o into {:?}", o.to_string());
    o.into()
}