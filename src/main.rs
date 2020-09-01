use std::env;
use std::fs;
use std::path;
use std::io::Write;

use std::collections::HashMap;

extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate markup;

mod data;
mod html;

use data::*;
use pest::Parser;


#[derive(Parser)]
#[grammar = "shb.pest"]
struct SHBParser;

pub struct Config{
    pub work_dir : path::PathBuf,
    pub shb_src_dir : path::PathBuf,
    pub song_out_dir : path::PathBuf
}

impl Config{
    fn new(args : &Vec<String>)-> Result<Config,&'static str>{

        let mut out = Config{
            shb_src_dir : path::PathBuf::new(),
            song_out_dir : path::PathBuf::new(),
            work_dir: path::PathBuf::new()
        };

        if args.len() < 2{
            return Err("No working directory given");
        }
        let path = path::PathBuf::from(&args[1]);

        if !path.is_dir() {
            return Err("Not a directory");
        }
        out.shb_src_dir.push("src/shb");
        out.song_out_dir.push("site/song");
        Ok(out)
    }
}

fn process_shb_folder(dir : &path::Path, out_html : &path::Path, out_bin : &path::Path)->Result<(),Box<dyn std::error::Error>>{
    if !out_html.is_dir() {
        return Err("Output directory not a directory")?;
    }
    if !out_bin.is_dir() {
        return Err("Output directory not a directory")?;
    }

    let entries = fs::read_dir(dir)?.filter_map(|p|{
        if let Ok(path) = p {
            let path = path.path();
            let p_ext = path.extension()?;
            if p_ext == std::ffi::OsStr::new("shb"){
                return Some(path);
            }
        }
        return None;
    });

    for entry in entries{
        let filename = entry.file_name();
        if let Some(fname) = filename.and_then(|f|f.to_str() ){
            print!("Parsing {} : ",fname);
        }else{
            print!("Parsing <Invalid utf-8 filename> : ");
        }
        let filename = filename.unwrap();
        let mut out_html = out_html.join(filename);
        out_html.set_extension("html");
        let mut out_bin = out_bin.join(filename);
        out_bin.set_extension("cbor");
        match process_shb_file(&entry, &out_html, &out_bin){
            Ok(()) => {println!("Ok")}
            Err(e) => {println!("Error:\n {}",e)}
        }
        

        /*
        let filename = entry.file_name().and_then(|f|f.to_str() );
        if let Some(fname) = filename{
            println!("Parsing {}",fname);
        }else{
            println!("Parsing <Invalid utf-8 filename>");
        }*/
    }
    return Ok(());
}

fn process_shb_file(file : &path::Path, out_html : &path::Path, out_bin : &path::Path)->Result<(),Box<dyn std::error::Error>>{
    let file_shb = fs::read_to_string(file)?;
    let mut tree = parse_shb(&file_shb)?;
    //tree.mut_chords(&|x| x.mut_transpose(1));
    let file_cbor = fs::File::create(out_bin)?;
    serde_cbor::to_writer(file_cbor, &tree)?;
    let mut file_html = fs::File::create(out_html)?;
    write!(file_html,"{}",html::SongPageHTML{song : &tree})?;
    return Ok(());
}

fn parse_shb<'i>(contents : &'i str)->Result<Song<'i>,pest::error::Error<Rule>>{
    let parse_tree = SHBParser::parse(Rule::song,contents)?;
    let tree = mk_song(parse_tree);
    return Ok( tree );
}

fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let args : Vec<String> = env::args().collect();

    if let Ok(config) = Config::new(&args){

    }

    process_shb_folder(path::Path::new("web/data/shb"), path::Path::new("web/song"), path::Path::new("web/data/cbor") )?;
    Ok(())
}







/*
fn chord_to_data(s : &str) -> Option<Chord> {
    let minor;
    match tonic_to_u8(s) {
        Some((tonic,mut char_indices)) => {
            match char_indices.next(){
                Some((i,'m')) => {minor = true;}
                _ => {char_indices.next_back();}
            }
            None
        },
        None => None
    }
}*/



//mk_* functions; they should panic only when there is a mismatch between tree structure
//as described and checked by the grammar; not the content.

fn mk_song (tree : pest::iterators::Pairs<Rule>) -> Song{
    let mut out = Song{
        name: "",
        tonic: 0,
        sections: Vec::new(),
        names: HashMap::new(),
        orders: HashMap::new(),
    };
    let mut default_order = Vec::new();
    for pair in tree {
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::meta_section => {
                    for meta_line in inner_pair.into_inner(){
                        let mut it = meta_line.into_inner();
                        let meta_id = it.next().unwrap().as_str();
                        let meta_val = it.next().unwrap().as_str();
                        if meta_id == "name" {
                            out.name = meta_val;
                        }else if meta_id == "tonic"{
                            if let Some(i) = tonic_to_u8(meta_val) {
                                out.tonic = i;
                            }
                        }
                    }
                },
                Rule::section => {
                    let new_section = mk_song_section(inner_pair);
                    if let Some(section_i_old) = out.names.get(new_section.id){
                        if new_section.lines.len() > 0 {
                            //TODO: Warn of existing section
                        }else{
                            default_order.push(*section_i_old);
                        }

                    }else{
                        //Register new section
                        let idx = out.sections.len();
                        out.names.insert(new_section.id.to_string(),idx);
                        default_order.push(idx);
                        out.sections.push(new_section);
                    }
                },//,
                Rule::EOI =>{}
                _ => unreachable!()
            };
        }
    }
    out
}
fn mk_song_section(tree : pest::iterators::Pair<Rule>) -> Section{
    let mut out = Section{
        id : "",
        name : "",
        delta_tonic : 0,
        lines : Vec::new()
    };

    for line_pair in tree.into_inner() {
        match line_pair.as_rule() {
            Rule::section_head => {
                let mut head_pair = line_pair.into_inner();
                if let Some(section_id) = head_pair.next() {
                    out.id = section_id.as_str();
                }
                if let Some(section_id) = head_pair.next() {
                    out.name = section_id.as_str();
                }
            },
            Rule::line_br => {out.lines.push(Line::Hr)},
            Rule::line => {out.lines.push(mk_song_line(line_pair));},
            _  => unreachable!()
        }
    }
    out
}

fn mk_song_line(tree : pest::iterators::Pair<Rule>) -> Line{
    let mut lyric_count:u32 = 0;
    let mut chord_count:u32 = 0;
    //1st pass: determine if it is a compound line
    'bar_loop : for bar_pair in (tree.clone()).into_inner() {
        for block_pair in bar_pair.into_inner(){
            for type_pairs in block_pair.into_inner(){
                
                match type_pairs.as_rule(){
                    Rule::lyric_block => {lyric_count+=1},
                    Rule::chord_block => {chord_count+=1},
                    Rule::empty_block => {},
                    _  => unreachable!()
                }
            }
            if lyric_count > 0 && chord_count > 0 { 
                break 'bar_loop;
            }
        }
    }

    //Compund case
    if lyric_count > 0 && chord_count > 0 {

        let mut line_data : Vec< Bar <( Vec<ChordItem>,& str)>> = Vec::new();
        for bar_pair in tree.into_inner() {

            let mut bar_data: Vec<(Vec<ChordItem>,&str)> = Vec::new();
            bar_data.reserve_exact(1); //CHECK: Most bars have only one block

            //Check empty-ness
            if let Some(Rule::empty_block) = bar_pair.clone().into_inner().next().and_then(|x| Some(x.as_rule()) ) {
                line_data.push(Bar::Empty);
                continue;
            }

            for block_pair in bar_pair.into_inner(){
                bar_data.push((Vec::new(),""));
                for type_pair in block_pair.into_inner(){
                    if let Some(top) = bar_data.last_mut(){
                        match type_pair.as_rule(){
                            Rule::lyric_block => {
                                top.1 = type_pair.as_str(); 
                            },
                            Rule::chord_block => {
                                top.0 = mk_chord_block_item(type_pair);
                            },
                            Rule::empty_block => {},
                            _  => unreachable!()
                        }
                    }
                }
             }
             line_data.push(Bar::Bar(bar_data));
        }

        return Line::Compound(line_data);

    } else if lyric_count > 0 {
        let mut line_data : Vec< Bar <& str> > = Vec::new();
        for bar_pair in tree.into_inner() {

            let mut bar_data: Vec<&str> = Vec::new();
            bar_data.reserve_exact(1); 


            if let Some(Rule::empty_block) = bar_pair.clone().into_inner().next().and_then(|x| Some(x.as_rule()) ) {
                line_data.push(Bar::Empty);
                continue;
            }

            for block_pair in bar_pair.into_inner(){
                for type_pair in block_pair.into_inner(){
                    bar_data.push(type_pair.as_str());
                }
             }
             line_data.push(Bar::Bar(bar_data));
        }

        return Line::Lyric(line_data);
    } else if chord_count > 0 {
        let mut line_data : Vec< Bar <Vec<ChordItem>> > = Vec::new();
        for bar_pair in tree.into_inner() {

            let mut bar_data: Vec<Vec<ChordItem>> = Vec::new();
            bar_data.reserve_exact(1); 


            if let Some(Rule::empty_block) = bar_pair.clone().into_inner().next().and_then(|x| Some(x.as_rule()) ) {
                line_data.push(Bar::Empty);
                continue;
            }

            for block_pair in bar_pair.into_inner(){
                for type_pair in block_pair.into_inner(){
                    bar_data.push(mk_chord_block_item(type_pair));
                }
             }
             line_data.push(Bar::Bar(bar_data));
        }

        return Line::Chord(line_data);
    } else { //This should not happen

    }
    return Line::Hr;
}

fn mk_chord_block_item (tree : pest::iterators::Pair<Rule>) -> Vec<ChordItem>{
    let mut out : Vec<ChordItem> = Vec::new();
    for chord_item in tree.into_inner(){
        let mono = match chord_item.as_rule(){
            Rule::parens => if chord_item.as_str() == "(" { ChordItem::ParensOpen}
                            else {ChordItem::ParensClose},
            Rule::nonchord => ChordItem::Nonmusic(chord_item.as_str()),
            Rule::melody => ChordItem::Melody(mk_melody(chord_item)),
            Rule::chord => ChordItem::Chord(mk_chord_symbol(chord_item)),
            _ => unreachable!()
        };
        out.push(mono);
    }
    out
}
fn mk_melody(tree : pest::iterators::Pair<Rule>) -> Vec<u8>{
    let mut out = Vec::new();
    for root in tree.into_inner(){
        if let Rule::root = root.as_rule(){
            out.push(tonic_to_u8(root.as_str()).unwrap_or_else(||panic!("Unreachable")) );
        }
    }
    out
}
fn mk_chord_symbol (tree : pest::iterators::Pair<Rule>) -> Chord{

    let mut out = Chord {
        root: 0,
        min: false,
        bass: 0,
        ext: Vec::new() //Dunno how to do set lifetime non obtusely :(
    };

    for chord_part in tree.into_inner(){
        match chord_part.as_rule(){
            Rule::root => {
                out.root = match tonic_to_u8(chord_part.as_str()){
                    Some(i) => i,
                    //Grammar isn't supposed to allow weirdness here
                    _ => panic!("Unhandled chord root name '{}'",chord_part.as_str())
                };
                out.bass = out.root;
            },
            Rule::min =>{
                out.min = true;
            },
            Rule::ext =>{
                for ext_part in chord_part.into_inner(){
                    match ext_part.as_rule(){
                        Rule::ext_keywords => out.ext.push( ChordExt::Keyword(ext_part.as_str())),
                        Rule::ext_unk_text => out.ext.push( ChordExt::Unknown(ext_part.as_str())),
                        Rule::ext_alt => out.ext.push( ChordExt::Alteration(ext_part.as_str()
                            .trim_matches(|c| c == '(' || c == ')')
                         )),
                        _ => unreachable!()
                    }
                }
            },
            Rule::bass => {
                let root_bass = chord_part.into_inner().next().unwrap_or_else(|| panic!("Bass note should always have one child"));
                out.bass = match tonic_to_u8(root_bass.as_str()){
                    Some(i) => i,
                    _ => panic!("Unhandled chord root name '{}'",root_bass.as_str())
                };
            },
            _ => unreachable!()
        }
    }
    out
}



