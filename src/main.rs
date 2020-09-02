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

mod shb{
    #[derive(Parser)]
    #[grammar = "shb.pest"]
    pub struct Parser;
}

mod lst{
    #[derive(Parser)]
    #[grammar = "lst.pest"]
    pub struct Parser;
}

pub struct Config{
    pub work_dir : path::PathBuf,
    pub shb_src_dir : path::PathBuf,
    pub lst_src_dir : path::PathBuf,
    pub list_out_dir : path::PathBuf,
    pub song_out_dir : path::PathBuf,
    pub bin_out_dir : path::PathBuf
}

impl Config{
    fn new(args : &Vec<String>)-> Result<Config,&'static str>{
        if args.len() < 2{
            return Err("No working directory given");
        }
        let mut out = Config{
            shb_src_dir : path::PathBuf::from(&args[1]),
            song_out_dir : path::PathBuf::from(&args[1]),
            list_out_dir : path::PathBuf::from(&args[1]),
            work_dir: path::PathBuf::from(&args[1]),
            lst_src_dir: path::PathBuf::from(&args[1]),
            bin_out_dir: path::PathBuf::from(&args[1])
        };
        
        if !out.shb_src_dir.is_dir() {
            return Err("Not a directory");
        }

        out.shb_src_dir.push("data/shb");
        out.bin_out_dir.push("data/cbor");
        out.song_out_dir.push("song");
        out.lst_src_dir.push("data/lst");
        out.list_out_dir.push("list");
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


fn process_lst_folder(dir : &path::Path, out_html : &path::Path, in_bin : &path::Path)->Result<(),Box<dyn std::error::Error>>{
    if !out_html.is_dir() {
        return Err("List output directory not a directory")?;
    }

    if !in_bin.is_dir() {
        return Err("Song include directory not a directory")?;
    }

    let entries = fs::read_dir(dir)?.filter_map(|p|{
        if let Ok(path) = p {
            let path = path.path();
            let p_ext = path.extension()?;
            if p_ext == std::ffi::OsStr::new("lst"){
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
        match process_lst_file(&entry, &out_html, &in_bin){
            Ok(()) => {println!("Ok")}
            Err(e) => {println!("Error:\n {}",e)}
        }
    }
    return Ok(());
}


fn process_shb_file(file : &path::Path, out_html : &path::Path, out_bin : &path::Path)->Result<(),Box<dyn std::error::Error>>{
    let file_shb = fs::read_to_string(file)?;
    let tree = parse_shb(&file_shb)?;
    //tree.mut_chords(&|x| x.mut_transpose(1));
    let file_cbor = fs::File::create(out_bin)?;
    serde_cbor::to_writer(file_cbor, &tree)?;
    let mut file_html = fs::File::create(out_html)?;
    write!(file_html,"{}",html::SongPageHTML{song : &tree})?;
    return Ok(());
}

fn process_lst_file(file : &path::Path, out_html : &path::Path, bin_dir : &path::Path)->Result<(),Box<dyn std::error::Error>>{
    let file_lst = fs::read_to_string(file)?;
    let list = parse_lst(&file_lst)?;
    let mut string_data = Vec::new();
    let mut tree_data : Vec<Song> = Vec::new();
    for item in &list{
        let string_file = fs::read(bin_dir.join(&item.id_file).with_extension("cbor"));
        string_data.push((string_file,item));
    }
    for song in &string_data{
        if let (Ok(song),entry) = song{
            let mut song : std::result::Result<Song, serde_cbor::error::Error> = serde_cbor::from_slice(song);
            if let Ok(mut song) = song{
                if let Some(tonality) = entry.tonic{
                    let delta = (tonality + 12 - song.tonic)%12;
                    song.mut_chords(&|x|{x.mut_transpose(delta);});
                }
                tree_data.push(song);
            }else{
                print!("Error parsing song cache");
            }
        }else if let (Err(e),entry) = song{
            print!("{} ",e);
        }
    }
    //tree.mut_chords(&|x| x.mut_transpose(1));
    //let file_cbor = fs::File::create(out_bin)?;
    //serde_cbor::to_writer(file_cbor, &tree)?;
    let mut file_html = fs::File::create(out_html)?;
    write!(file_html,"{}",html::SonglistPageHTML{songlist : &tree_data})?;
    return Ok(());
}

fn parse_shb<'i>(contents : &'i str)->Result<Song<'i>,pest::error::Error<shb::Rule>>{
    let parse_tree = shb::Parser::parse(shb::Rule::song,contents)?;
    let tree = mk_song(parse_tree);
    return Ok( tree );
}

fn parse_lst<'i>(contents : &'i str)->Result<Vec<SonglistEntry>,pest::error::Error<lst::Rule>>{
    let parse_tree = lst::Parser::parse(lst::Rule::list,contents)?;
    let tree = mk_song_list(parse_tree);
    return Ok( tree );
}


fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let args : Vec<String> = env::args().collect();
    if let Ok(config) = Config::new(&args){
        process_shb_folder(&config.shb_src_dir, &config.song_out_dir, &config.bin_out_dir )?;
        process_lst_folder(&config.lst_src_dir, &config.list_out_dir, &config.bin_out_dir )?;
    }

        Ok(())
}







//mk_* functions; they should panic only when there is a mismatch between tree structure
//as described and checked by the grammar; not the content.
fn mk_song_list(tree : pest::iterators::Pairs<lst::Rule>) -> Vec<SonglistEntry>{
    let mut out = Vec::new();
    for pair in tree{
        for line in pair.into_inner(){
            match line.as_rule(){
                    lst::Rule::line => {
                        let mut iter = line.into_inner();
                        let name = iter.next().unwrap().as_str();
                        let mut entry = SonglistEntry{
                            id_file : String::from(name),
                            order : None,
                            tonic : None
                        };
                        for desc in iter{
                            let mut iter_desc = desc.into_inner();
                            let name_desc = iter_desc.next().unwrap().as_str();
                            let val_desc = iter_desc.next().unwrap().as_str();
                            match name_desc{
                                "tonic" | "tone" | "key" => {
                                    entry.tonic = tonality_to_u8(val_desc);
                                    if let None = entry.tonic {
                                        println!("Warning, unrecognized key")
                                    }
                                },
                                "order" | "sections" => {
                                    //TODO: implement ordering
                                },
                                _ => ()
                            }
                        }
                        out.push(entry);
                   
                }
                _ => {}
            }
        }
    }
    out
}

fn mk_song (tree : pest::iterators::Pairs<shb::Rule>) -> Song{
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
                shb::Rule::meta_section => {
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
                shb::Rule::section => {
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
                shb::Rule::EOI =>{}
                _ => unreachable!()
            };
        }
    }
    out
}

fn mk_song_section(tree : pest::iterators::Pair<shb::Rule>) -> Section{
    let mut out = Section{
        id : "",
        name : "",
        delta_tonic : 0,
        lines : Vec::new()
    };

    for line_pair in tree.into_inner() {
        match line_pair.as_rule() {
            shb::Rule::section_head => {
                let mut head_pair = line_pair.into_inner();
                if let Some(section_id) = head_pair.next() {
                    out.id = section_id.as_str();
                }
                if let Some(section_id) = head_pair.next() {
                    out.name = section_id.as_str();
                }
            },
            shb::Rule::line_br => {out.lines.push(Line::Hr)},
            shb::Rule::line => {out.lines.push(mk_song_line(line_pair));},
            _  => unreachable!()
        }
    }
    out
}

fn mk_song_line(tree : pest::iterators::Pair<shb::Rule>) -> Line{
    let mut lyric_count:u32 = 0;
    let mut chord_count:u32 = 0;
    //1st pass: determine if it is a compound line
    'bar_loop : for bar_pair in (tree.clone()).into_inner() {
        for block_pair in bar_pair.into_inner(){
            for type_pairs in block_pair.into_inner(){
                
                match type_pairs.as_rule(){
                    shb::Rule::lyric_block => {lyric_count+=1},
                    shb::Rule::chord_block => {chord_count+=1},
                    shb::Rule::empty_block => {},
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
            if let Some(shb::Rule::empty_block) = bar_pair.clone().into_inner().next().and_then(|x| Some(x.as_rule()) ) {
                line_data.push(Bar::Empty);
                continue;
            }

            for block_pair in bar_pair.into_inner(){
                bar_data.push((Vec::new(),""));
                for type_pair in block_pair.into_inner(){
                    if let Some(top) = bar_data.last_mut(){
                        match type_pair.as_rule(){
                            shb::Rule::lyric_block => {
                                top.1 = type_pair.as_str(); 
                            },
                            shb::Rule::chord_block => {
                                top.0 = mk_chord_block_item(type_pair);
                            },
                            shb::Rule::empty_block => {},
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


            if let Some(shb::Rule::empty_block) = bar_pair.clone().into_inner().next().and_then(|x| Some(x.as_rule()) ) {
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


            if let Some(shb::Rule::empty_block) = bar_pair.clone().into_inner().next().and_then(|x| Some(x.as_rule()) ) {
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
        unreachable!();
    }
}

fn mk_chord_block_item (tree : pest::iterators::Pair<shb::Rule>) -> Vec<ChordItem>{
    let mut out : Vec<ChordItem> = Vec::new();
    for chord_item in tree.into_inner(){
        let mono = match chord_item.as_rule(){
            shb::Rule::parens => if chord_item.as_str() == "(" { ChordItem::ParensOpen}
                            else {ChordItem::ParensClose},
            shb::Rule::nonchord => ChordItem::Nonmusic(chord_item.as_str()),
            shb::Rule::melody => ChordItem::Melody(mk_melody(chord_item)),
            shb::Rule::chord => ChordItem::Chord(mk_chord_symbol(chord_item)),
            _ => unreachable!()
        };
        out.push(mono);
    }
    out
}
fn mk_melody(tree : pest::iterators::Pair<shb::Rule>) -> Vec<u8>{
    let mut out = Vec::new();
    for root in tree.into_inner(){
        if let shb::Rule::root = root.as_rule(){
            out.push(tonic_to_u8(root.as_str()).unwrap_or_else(||panic!("Unreachable")) );
        }
    }
    out
}
fn mk_chord_symbol (tree : pest::iterators::Pair<shb::Rule>) -> Chord{

    let mut out = Chord {
        root: 0,
        min: false,
        bass: 0,
        ext: Vec::new()
    };

    for chord_part in tree.into_inner(){
        match chord_part.as_rule(){
            shb::Rule::root => {
                out.root = match tonic_to_u8(chord_part.as_str()){
                    Some(i) => i,
                    //Grammar isn't supposed to allow weirdness here
                    _ => panic!("Unhandled chord root name '{}'",chord_part.as_str())
                };
                out.bass = out.root;
            },
            shb::Rule::min =>{
                out.min = true;
            },
            shb::Rule::ext =>{
                for ext_part in chord_part.into_inner(){
                    match ext_part.as_rule(){
                        shb::Rule::ext_keywords => out.ext.push( ChordExt::Keyword(ext_part.as_str())),
                        shb::Rule::ext_unk_text => out.ext.push( ChordExt::Unknown(ext_part.as_str())),
                        shb::Rule::ext_alt => out.ext.push( ChordExt::Alteration(ext_part.as_str()
                            .trim_matches(|c| c == '(' || c == ')')
                         )),
                        _ => unreachable!()
                    }
                }
            },
            shb::Rule::bass => {
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



