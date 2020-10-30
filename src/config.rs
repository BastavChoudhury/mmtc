use serde::{
    de::{self, EnumAccess, SeqAccess, VariantAccess, Visitor},
    Deserialize, Deserializer,
};

use std::{
    cmp::min,
    error,
    fmt::{self, Formatter},
};

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "fps_default")]
    pub fps: f64,
    #[serde(default = "ups_default")]
    pub ups: f64,
    pub layout: Widget,
}

fn fps_default() -> f64 {
    30.0
}

fn ups_default() -> f64 {
    4.0
}

#[derive(Debug, Deserialize)]
pub enum Widget {
    Rows(Vec<Constrained<Widget>>),
    Columns(Vec<Constrained<Widget>>),
    Textbox(Texts),
    Queue { columns: Vec<Constrained<Texts>> },
}

#[derive(Debug, Deserialize)]
pub enum Constrained<T> {
    Free(T),
    Fixed(u16, T),
    Ratio(u32, T),
}

#[derive(Debug)]
pub enum Texts {
    Empty,
    Plain(String),
    CurrentElapsed,
    CurrentDuration,
    CurrentFile,
    CurrentTitle,
    CurrentArtist,
    CurrentAlbum,
    QueueDuration,
    QueueFile,
    QueueTitle,
    QueueArtist,
    QueueAlbum,
    Parts(Vec<Texts>),
    If(Condition, Box<Texts>, Box<Texts>),
}

#[derive(Debug, Deserialize)]
pub enum Condition {
    Playing,
    TitleExist,
    ArtistExist,
    AlbumExist,
}

impl<'de> Deserialize<'de> for Texts {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TextsVisitor;
        impl<'de> Visitor<'de> for TextsVisitor {
            type Value = Texts;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("enum Texts")
            }

            fn visit_unit<E: error::Error>(self) -> Result<Self::Value, E> {
                Ok(Texts::Empty)
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut sa: A) -> Result<Self::Value, A::Error> {
                let mut xs = Vec::with_capacity(min(sa.size_hint().unwrap_or(0), 4096));
                while let Some(x) = sa.next_element()? {
                    xs.push(x);
                }
                Ok(Texts::Parts(xs))
            }

            fn visit_enum<A: EnumAccess<'de>>(self, ea: A) -> Result<Self::Value, A::Error> {
                #[derive(Deserialize)]
                #[serde(field_identifier)]
                enum Variant {
                    Plain,
                    CurrentElapsed,
                    CurrentDuration,
                    CurrentFile,
                    CurrentTitle,
                    CurrentArtist,
                    CurrentAlbum,
                    QueueDuration,
                    QueueFile,
                    QueueTitle,
                    QueueArtist,
                    QueueAlbum,
                    Parts,
                    If,
                    IfNot,
                }

                struct IfVisitor;
                impl<'de> Visitor<'de> for IfVisitor {
                    type Value = Texts;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        formatter.write_str("IfNot variant")
                    }

                    fn visit_seq<A: SeqAccess<'de>>(
                        self,
                        mut sa: A,
                    ) -> Result<Self::Value, A::Error> {
                        Ok(Texts::If(
                            sa.next_element()?
                                .ok_or_else(|| de::Error::invalid_length(0, &self))?,
                            sa.next_element()?.map_or_else(
                                || Err(de::Error::invalid_length(1, &self)),
                                |x| Ok(Box::new(x)),
                            )?,
                            Box::new(sa.next_element()?.unwrap_or(Texts::Empty)),
                        ))
                    }
                }

                struct IfNotVisitor;
                impl<'de> Visitor<'de> for IfNotVisitor {
                    type Value = Texts;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        formatter.write_str("IfNot variant")
                    }

                    fn visit_seq<A: SeqAccess<'de>>(
                        self,
                        mut sa: A,
                    ) -> Result<Self::Value, A::Error> {
                        let cond = sa
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        let no = sa.next_element()?.map_or_else(
                            || Err(de::Error::invalid_length(1, &self)),
                            |x| Ok(Box::new(x)),
                        )?;
                        let yes = Box::new(sa.next_element()?.unwrap_or(Texts::Empty));
                        Ok(Texts::If(cond, yes, no))
                    }
                }

                let (variant, va) = ea.variant()?;

                macro_rules! unit_variant {
                    ($v:ident) => {{
                        va.unit_variant()?;
                        Ok(Texts::$v)
                    }};
                }

                match variant {
                    Variant::Plain => Ok(Texts::Plain(va.newtype_variant()?)),
                    Variant::CurrentElapsed => unit_variant!(CurrentElapsed),
                    Variant::CurrentDuration => unit_variant!(CurrentDuration),
                    Variant::CurrentFile => unit_variant!(CurrentFile),
                    Variant::CurrentTitle => unit_variant!(CurrentTitle),
                    Variant::CurrentArtist => unit_variant!(CurrentArtist),
                    Variant::CurrentAlbum => unit_variant!(CurrentAlbum),
                    Variant::QueueDuration => unit_variant!(QueueDuration),
                    Variant::QueueFile => unit_variant!(QueueFile),
                    Variant::QueueTitle => unit_variant!(QueueTitle),
                    Variant::QueueArtist => unit_variant!(QueueArtist),
                    Variant::QueueAlbum => unit_variant!(QueueAlbum),
                    Variant::Parts => Ok(Texts::Parts(va.newtype_variant()?)),
                    Variant::If => va.tuple_variant(3, IfVisitor),
                    Variant::IfNot => va.tuple_variant(3, IfNotVisitor),
                }
            }
        }

        de.deserialize_enum(
            "Texts",
            &[
                "Plain",
                "CurrentElapsed",
                "CurrentDuration",
                "CurrentFile",
                "CurrentTitle",
                "CurrentArtist",
                "CurrentAlbum",
                "QueueDuration",
                "QueueFile",
                "QueueTitle",
                "QueueArtist",
                "QueueAlbum",
                "Parts",
                "If",
                "IfNot",
            ],
            TextsVisitor,
        )
    }
}
