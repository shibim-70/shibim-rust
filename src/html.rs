
use crate::data::*;
//Its an horrendous mess, but hadn't I used this macro-ing, it would have taken ages
markup::define!{
    SongPageHTML<'i>(song: &'i Song<'i>){
        {markup::doctype()}
        html {
            head{
                meta [charset ="utf-8"];
                meta [name="viewport", content="width=device-width, initial-scale=1.0"];
                link [rel="stylesheet", href="../css/style.css"];
            }
            body{
                {SongHTML{song : song}}
            }
        }
    }
    SongHTML<'i>(song: &'i Song<'i>){
        article ."u-song" ["data-tonic" = song.tonic]{
            div ."u-song-title-box" {
                h2 { {song.name} }
            }
            @for section in &song.sections{
                {SectionHTML{section : section}}
            }
        }
    }
    SectionHTML<'i>(section: &'i Section<'i>){
        section ."u-section" ["data-id" = section.id]{
            div ."u-section-title-box"{
                div ."u-section-title-background"{
                    span ."u-section-id" { {section.id} }
                    @if !section.name.is_empty(){
                        h3{
                            {{section.name}}
                        }
                    }
                }
            }
            @for line in &section.lines{
                div ."u-l"{
                    {LineHTML{line : line } }
                }
            }
        }
    }
    LineHTML<'i>(line : &'i Line<'i>){
        @match line{
            Line::Hr => { hr; }
            Line::Lyric(vec) => {
                @for bar in vec{
                    span ."u-b" {
                        @if let Bar::Bar(blocks) = bar {
                            span ."u-o" {
                                pre {{blocks.join("")}}
                            }
                        }else{
                            span ."u-o"{}
                        }
                    }
                }
            }
            Line::Chord(vec) =>{
                @for bar in vec{
                    span ."u-b" {
                        @if let Bar::Bar(blocks) = bar {
                            span ."u-o" {
                                @for block in blocks{
                                    mark{
                                        { ChordBlockHTML{ chord_block : block } }
                                    }
                                }
                            }
                        }else{
                            span ."u-o"{}
                        }
                    }
                }
            }
            Line::Compound(vec) => {
                @for bar in vec{
                    span ."u-b"{
                        @if let Bar::Bar(blocks) = bar {
                            @for block in blocks{
                                    span ."u-o" {
                                    mark{
                                        { ChordBlockHTML{ chord_block : &block.0 } }
                                    }
                                    pre{
                                        {block.1}
                                    }
                                    }
                            }
                        }else{
                            span ."u-o"{}
                        }
                    }
                }
            }
        }
    }

    ChordHTML<'i>(chord : &'i Chord<'i>){
        u .r { {markup::raw(u8_to_tonic_html(chord.root,true))} }
        @if chord.min {
            {"m"}
        }

        @for ext in &chord.ext{
            @match ext{
                ChordExt::Alteration(s) => { sup{ {s} } }
                ChordExt::Keyword(s) => { 
                    @match *s{
                        "M" | "Maj" | "maj" =>{
                            {"Δ"}
                        }
                        "sus4" | "sus2" | "13" | "11"=>{
                            span .c { {s} }
                        }
                        _ => { {s} }
                    }
                }//TODO compact Maj sus* 
                ChordExt::Unknown(_) => {}
            }
        }

        @if chord.bass != chord.root{
            sub { "/" u .r {{markup::raw(u8_to_tonic_html(chord.bass,false))}} }
        }
    }

    ChordBlockHTML<'i>(chord_block : &'i Vec<ChordItem<'i>>){
        //We need some look-up to properly do parens
        @for chord_item_i in 0..chord_block.len(){
            @let chord_item = &chord_block[chord_item_i];
            //'Tis ugly
            @let antecedes_parens =
             if let Some(chord_next) = chord_block.get(chord_item_i+1){
                 matches!(chord_next,ChordItem::ParensClose)
                } else {false};
            @match chord_item{
                ChordItem::Chord(c) => { {ChordHTML{ chord: c} } @if !antecedes_parens {" "} }
                ChordItem::Melody(_v) => { /*TODO*/ }
                ChordItem::Nonmusic(s) => { {s}  @if !antecedes_parens {" "} }
                ChordItem::ParensClose => { "("  }
                ChordItem::ParensOpen => { ")" }
            }
        }
    }

    
}
