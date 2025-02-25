use std::hash::Hash;
use std::marker::PhantomData;

use crate::field::LurkField;
use crate::tag::{ContTag, ExprTag};

/// The internal untagged raw Store pointer
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RawPtr {
    /// Null is used to represent ZPtrs with hash digests of F::zero()
    /// currently only ZExpr::StrNil and ZExpr::SymNil
    Null,
    /// Opaque represents pointers to expressions whose hashes are known, but
    /// whose preimages are unknown
    Opaque(usize),
    /// Index represents a pointer into one of several possible `IndexSet`s in `Store`.
    /// The specific IndexSet is determined by the `Ptr` `tag` field.
    Index(usize),
}

impl RawPtr {
    /// Construct a new RawPtr, with default `Index` (as the most common)`
    pub fn new(p: usize) -> Self {
        Self::Index(p)
    }

    /// check if a RawPtr is Opaque
    pub const fn is_opaque(&self) -> bool {
        matches!(self, Self::Opaque(_))
    }

    /// check if a RawPtr is Null
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// get the index of an Opaque RawPtr
    pub fn opaque_idx(&self) -> Option<usize> {
        match self {
            Self::Opaque(x) => Some(*x),
            _ => None,
        }
    }

    /// get the index of a RawPtr
    pub fn idx(&self) -> Option<usize> {
        match self {
            Self::Index(x) => Some(*x),
            _ => None,
        }
    }
}

/// A `Store` pointer
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Ptr<F: LurkField> {
    /// An expression tag
    pub tag: ExprTag,
    /// The underlying pointer, which can be null, opaque, or an index
    pub raw: RawPtr,
    /// PhantomData is needed to consume the `F: LurkField` parameter, since
    /// we want to pin our Ptr to a specific field (even though we don't
    /// actually use it)
    pub _f: PhantomData<F>,
}

#[allow(clippy::derived_hash_with_manual_eq)]
impl<F: LurkField> Hash for Ptr<F> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tag.hash(state);
        self.raw.hash(state);
    }
}

impl<F: LurkField> Ptr<F> {
    // TODO: Make these methods and the similar ones defined on expression consistent, probably including a shared trait.

    // NOTE: Although this could be a type predicate now, when NIL becomes a symbol, it won't be possible.
    /// check if a Ptr is `Nil` pointer
    pub const fn is_nil(&self) -> bool {
        matches!(self.tag, ExprTag::Nil)
        // FIXME: check value also, probably
    }

    /// check if a Ptr is a `Cons` pointer
    pub const fn is_cons(&self) -> bool {
        matches!(self.tag, ExprTag::Cons)
    }

    // TODO: Is this still needed?
    /// check if a Ptr is atomic pointer
    pub const fn is_atom(&self) -> bool {
        !self.is_cons()
    }

    // check if a Ptr is a list pointer
    pub const fn is_list(&self) -> bool {
        matches!(self.tag, ExprTag::Nil | ExprTag::Cons)
    }

    /// check if a Ptr is an opaque pointer
    pub const fn is_opaque(&self) -> bool {
        self.raw.is_opaque()
    }

    // TODO: Is this still needed?
    pub const fn as_cons(self) -> Option<Self> {
        if self.is_cons() {
            Some(self)
        } else {
            None
        }
    }

    // TODO: Is this still needed?
    pub const fn as_list(self) -> Option<Self> {
        if self.is_list() {
            Some(self)
        } else {
            None
        }
    }

    /// Construct a Ptr from an index
    pub fn index(tag: ExprTag, idx: usize) -> Self {
        Ptr {
            tag,
            raw: RawPtr::Index(idx),
            _f: Default::default(),
        }
    }

    /// Construct a Ptr from an opaque index
    pub fn opaque(tag: ExprTag, idx: usize) -> Self {
        Ptr {
            tag,
            raw: RawPtr::Opaque(idx),
            _f: Default::default(),
        }
    }

    /// Construct a null Ptr
    pub fn null(tag: ExprTag) -> Self {
        Ptr {
            tag,
            raw: RawPtr::Null,
            _f: Default::default(),
        }
    }

    #[inline]
    pub fn cast(self, tag: ExprTag) -> Self {
        Ptr {
            tag,
            raw: self.raw,
            _f: self._f,
        }
    }
}

impl<F: LurkField> From<char> for Ptr<F> {
    fn from(c: char) -> Self {
        Self {
            tag: ExprTag::Char,
            raw: RawPtr::Index(u32::from(c) as usize),
            _f: Default::default(),
        }
    }
}

/// A pointer to a continuation. Logically this is the same a Ptr and should
/// probably be combined with it in a future refactor
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ContPtr<F: LurkField> {
    pub tag: ContTag,
    pub raw: RawPtr,
    pub _f: PhantomData<F>,
}

#[allow(clippy::derived_hash_with_manual_eq)]
impl<F: LurkField> Hash for ContPtr<F> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tag.hash(state);
        self.raw.hash(state);
    }
}

impl<F: LurkField> ContPtr<F> {
    pub fn new(tag: ContTag, raw: RawPtr) -> Self {
        Self {
            tag,
            raw,
            _f: Default::default(),
        }
    }
    pub const fn is_error(&self) -> bool {
        matches!(self.tag, ContTag::Error)
    }

    pub fn index(tag: ContTag, idx: usize) -> Self {
        ContPtr {
            tag,
            raw: RawPtr::Index(idx),
            _f: Default::default(),
        }
    }

    pub fn opaque(tag: ContTag, idx: usize) -> Self {
        ContPtr {
            tag,
            raw: RawPtr::Index(idx),
            _f: Default::default(),
        }
    }

    pub fn null(tag: ContTag) -> Self {
        ContPtr {
            tag,
            raw: RawPtr::Null,
            _f: Default::default(),
        }
    }
}

pub trait TypePredicates {
    fn is_fun(&self) -> bool;
    fn is_self_evaluating(&self) -> bool;
    fn is_potentially(&self, tag: ExprTag) -> bool;
}

impl<F: LurkField> TypePredicates for Ptr<F> {
    fn is_fun(&self) -> bool {
        self.tag.is_fun()
    }
    fn is_self_evaluating(&self) -> bool {
        self.tag.is_self_evaluating()
    }
    fn is_potentially(&self, tag: ExprTag) -> bool {
        self.tag.is_potentially(tag)
    }
}
