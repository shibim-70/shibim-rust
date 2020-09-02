
use std::collections::HashMap;
use serde_derive::{Deserialize, Serialize};

//const NUM_TONIC_VALUES:[u8;7] = [0,2,4,5,7,9,11];
const CHAR_TONIC_VALUES:[u8;7] = [9,11,0,2,4,5,7];

const SHARP_TONIC_NAMES:[&str;12] = ["C","C#","D","D#","E","F","F#","G","G#","A","A#","B"];
const FLAT_TONIC_NAMES:[&str;12] = ["C","Db","D","Eb","E","F","Gb","G","Ab","A","Bb","B"];


const SHARP_TONIC_NAMES_HTML:[&str;12] = ["C","C<sup>#</sup>","D","D<sup>#</sup>","E",
                                        "F","F<sup>#</sup>","G","G<sup>#</sup>","A","A<sup>#</sup>","B"];
const FLAT_TONIC_NAMES_HTML:[&str;12] = ["C","D<sup>b</sup>","D","E<sup>b</sup>","E","F","G<sup>b</sup>",
                                        "G","A<sup>b</sup>","A","B<sup>b</sup>","B"];

                                    
//To summarise this mess:
//song -> [section]
//section -> [lines]
//lines -> [Bar<str>] | [Bar<ChordItems>]|[Bar<(str, [ChordItem] )>] | line_break
//Bar<T> -> [T] | empty  //May replace this confusing one
//ChordItem -> Chord | Melody | ...
//Melody -> [ u8 ]


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "X")]
pub enum ChordExt<'i>{
    #[serde(rename = "k")] 
    Keyword(&'i str),
    #[serde(rename = "a")] 
    Alteration(&'i str),
    #[serde(rename = "u")] 
    Unknown(&'i str),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "C")]
pub struct Chord<'i>{
    #[serde(rename = "r")] 
    pub root: u8,
    #[serde(rename = "b")] 
    pub bass: u8,
    #[serde(rename = "m")] 
    pub min: bool,
    #[serde(borrow)]
    #[serde(rename = "x")] 
    pub ext: Vec<ChordExt<'i>>,
    pub time: Option<Time>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "t")]
pub struct Time{
    #[serde(rename = "r")] 
    pub beat: u8,
    pub num: u8,
    pub den: u8,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "c")]
pub enum ChordItem<'i>{
    #[serde(rename = "h")] 
    Chord(Chord<'i>),
    #[serde(rename = "(")] 
    ParensOpen,
    #[serde(rename = ")")] 
    ParensClose,
    #[serde(rename = "q")] 
    Nonmusic(&'i str),
    #[serde(rename = "y")] 
    Melody(Vec<u8>)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "B")]
pub enum Bar<T>{
    #[serde(rename = ".")] 
    Empty,
    #[serde(rename = "|")] 
    Bar(Vec< T >)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "l")]
pub enum Line<'i>{
    #[serde(rename = "!")] 
    Hr,
    #[serde(rename = "~")] 
    Lyric(Vec<Bar<&'i str>>),
    #[serde(rename = "*")] 
    Chord(Vec<Bar< Vec<ChordItem<'i>> >>),
    //(Chord, Lyrics)
    #[serde(borrow)]
    #[serde(rename = ",")] 
    Compound(Vec<Bar<(Vec<ChordItem<'i>>,&'i str)>>)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "s")]
pub struct Section<'i>{
    pub id: &'i str,
    #[serde(rename = "na")] 
    pub name: &'i str,
    #[serde(rename = "de")] 
    pub delta_tonic: u8,
    #[serde(rename = "li")] 
    pub lines: Vec< Line<'i> >
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "S")]
pub struct Song<'i>{
    pub name: &'i str,
    pub tonic: u8,
    pub sections: Vec< Section<'i> >,
    pub names: HashMap<String, usize>,
    pub orders: HashMap<String, Vec< usize > >
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SonglistEntry{
    pub id_file : String,
    pub tonic : Option<u8>,
    pub order : Option<Vec<String>>,
}

pub struct IndexEntry{
    pub path : String,
    pub name : String
}

//TODO, clean up these two functions
pub fn tonic_to_u8 (s : &str) -> Option<u8>{
    let mut chars = s.char_indices();
    let mut n;
    match chars.next(){
        
        Some((_,ch)) =>{
            
            if ch >= 'A' && ch <= 'G'{
                n = CHAR_TONIC_VALUES[((ch as usize) - ('A' as usize) )%7];
            }else{
                return None;
            }
        }
        None => return None
    }
    match chars.next(){
        Some((_,ch)) =>{
            match ch{
                '#' => {n = (n + 1)%12},
                'b' => {n = (n+11)%12}
                _ => {chars.next_back();}
            }
        }
        None => {}
    }
    return Some(n as u8);
}

pub fn tonality_to_u8(s : &str) -> Option<u8>{
    let base = tonic_to_u8(&s.to_ascii_uppercase());
    if let Some(i) = base{
        let mut c = s.chars();
        let mut min : bool = false;

        if matches!(c.next(),Some('m')){
            min = true;
        }
        if matches!(c.next(),Some('m')){
            min = true;
        }
        if min { Some ((i + 3) %12)} else { Some(i) }
    }else{
        None
    }

}

pub fn tonality_default_sharp(t:u8)->bool{
    match t{
        5 | 10 | 3 | 8 => false,
        _  => true
    }
}

pub fn u8_to_tonic (i: u8, sharp: bool) -> &'static str {
    let norm = (i % 12) as usize;
    if sharp{
        SHARP_TONIC_NAMES[norm]
    }else{
        FLAT_TONIC_NAMES[norm]
    }
}

pub fn u8_to_tonic_html (i: u8, sharp: bool) -> &'static str {
    let norm = (i % 12) as usize;
    if sharp{
        SHARP_TONIC_NAMES_HTML[norm]
    }else{
        FLAT_TONIC_NAMES_HTML[norm]
    }
}
impl ChordItem<'_>{
    pub fn mut_transpose(&mut self , am : u8){
        match self{
            ChordItem::Chord(chord_item) =>{
                chord_item.bass = (chord_item.bass + am) % 12;
                chord_item.root = (chord_item.root +am ) % 12;
            }
            ChordItem::Melody(notes) => {
                for note in notes.iter_mut(){
                    *note = (*note + am) % 12;
                }
            }
            _ => ()
        }
    }
}
impl Song<'_>{
    pub fn mut_chords(&mut self, f : &dyn Fn(&mut ChordItem)){
        for sect in &mut self.sections{
            for line in &mut sect.lines{
                match line{
                    Line::Chord(c_line) => {
                        for c_item in c_line.iter_mut(){
                            if let Bar::Bar(c_item) = c_item{
                                c_item.iter_mut().for_each(|x|{
                                    x.iter_mut().for_each(|x| f(x) );
                                });
                            }
                        }
                    } ,
                    Line::Compound(co_line) => {
                        for bar in co_line.iter_mut(){
                            if let Bar::Bar(bar) = bar {
                                for (items, _) in bar{
                                    items.iter_mut().for_each(|x| f(x));
                                }
                            }
                        }
                    },
                    Line::Hr | Line::Lyric(_) => {}
                }
            }
        }

    }
}