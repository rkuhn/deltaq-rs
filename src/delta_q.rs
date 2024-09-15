use crate::CDF;
use std::fmt::{self, Display};

/// A DeltaQ is a representation of a probability distribution that can be
/// manipulated in various ways.
///
/// The Display implementation prints out the expression using the syntax from the paper:
/// - Names are printed as-is.
/// - CDFs are printed as-is.
/// - Sequences are printed as `A •->-• B`.
/// - Choices are printed as `A a⇌b B`.
/// - Universal quantifications are printed as `∀(A|B)`.
/// - Existential quantifications are printed as `∃(A|B)`.
#[derive(Debug, Clone, PartialEq)]
pub enum DeltaQ {
    /// A named DeltaQ that can be referenced elsewhere.
    Name(String),
    /// A CDF that is used as a DeltaQ.
    CDF(CDF),
    /// The convolution of two DeltaQs, describing the sequential execution of two outcomes.
    Seq(Box<DeltaQ>, Box<DeltaQ>),
    /// A choice between two DeltaQs (i.e. their outcomes), with a given weight of each.
    Choice(Box<DeltaQ>, f64, Box<DeltaQ>, f64),
    /// A DeltaQ that is the result of a universal quantification over two DeltaQs,
    /// meaning that both outcomes must occur.
    ForAll(Box<DeltaQ>, Box<DeltaQ>),
    /// A DeltaQ that is the result of an existential quantification over two DeltaQs,
    /// meaning that at least one of the outcomes must occur.
    ForSome(Box<DeltaQ>, Box<DeltaQ>),
}

impl Display for DeltaQ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, false)
    }
}

impl DeltaQ {
    /// Create a new DeltaQ from a name, referencing a variable.
    pub fn name(name: &str) -> DeltaQ {
        DeltaQ::Name(name.to_string())
    }

    /// Create a new DeltaQ from a CDF.
    pub fn cdf(cdf: CDF) -> DeltaQ {
        DeltaQ::CDF(cdf)
    }

    /// Create a new DeltaQ from the convolution of two DeltaQs.
    pub fn seq(first: DeltaQ, second: DeltaQ) -> DeltaQ {
        DeltaQ::Seq(Box::new(first), Box::new(second))
    }

    /// Create a new DeltaQ from a choice between two DeltaQs.
    pub fn choice(first: DeltaQ, first_weight: f64, second: DeltaQ, second_weight: f64) -> DeltaQ {
        DeltaQ::Choice(
            Box::new(first),
            first_weight,
            Box::new(second),
            second_weight,
        )
    }

    /// Create a new DeltaQ from a universal quantification over two DeltaQs.
    pub fn for_all(first: DeltaQ, second: DeltaQ) -> DeltaQ {
        DeltaQ::ForAll(Box::new(first), Box::new(second))
    }

    /// Create a new DeltaQ from an existential quantification over two DeltaQs.
    pub fn for_some(first: DeltaQ, second: DeltaQ) -> DeltaQ {
        DeltaQ::ForSome(Box::new(first), Box::new(second))
    }

    fn display(&self, f: &mut fmt::Formatter<'_>, parens: bool) -> fmt::Result {
        match self {
            DeltaQ::Name(name) => {
                write!(f, "{}", name)
            }
            DeltaQ::CDF(cdf) => {
                write!(f, "{:?}", cdf)
            }
            DeltaQ::Seq(first, second) => {
                if parens {
                    write!(f, "(")?;
                }
                first.display(f, true)?;
                write!(f, " •->-• ")?;
                second.display(f, true)?;
                if parens {
                    write!(f, ")")?;
                }
                Ok(())
            }
            DeltaQ::Choice(first, first_weight, second, second_weight) => {
                if parens {
                    write!(f, "(")?;
                }
                first.display(f, true)?;
                write!(f, " {}⇌{} ", first_weight, second_weight)?;
                second.display(f, true)?;
                if parens {
                    write!(f, ")")?;
                }
                Ok(())
            }
            DeltaQ::ForAll(first, second) => {
                write!(f, "∀({} | {})", first, second)
            }
            DeltaQ::ForSome(first, second) => {
                write!(f, "∃({} | {})", first, second)
            }
        }
    }

    pub fn eval(&self) -> Result<CDF, &'static str> {
        match self {
            DeltaQ::Name(_) => Err("Cannot evaluate a name"),
            DeltaQ::CDF(cdf) => Ok(cdf.clone()),
            DeltaQ::Seq(first, second) => {
                let first_cdf = first.eval()?;
                let second_cdf = second.eval()?;
                first_cdf.convolve(&second_cdf)
            }
            DeltaQ::Choice(first, first_fraction, second, second_fraction) => {
                let first_cdf = first.eval()?;
                let second_cdf = second.eval()?;
                first_cdf.choice(
                    *first_fraction / (*first_fraction + *second_fraction),
                    &second_cdf,
                )
            }
            DeltaQ::ForAll(first, second) => {
                let first_cdf = first.eval()?;
                let second_cdf = second.eval()?;
                first_cdf.for_all(&second_cdf)
            }
            DeltaQ::ForSome(first, second) => {
                let first_cdf = first.eval()?;
                let second_cdf = second.eval()?;
                first_cdf.for_some(&second_cdf)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_name() {
        let dq = DeltaQ::name("A");
        assert_eq!(dq.to_string(), "A");
    }

    #[test]
    fn test_display_cdf() {
        let cdf = CDF::new(vec![0.0, 0.2, 0.9], 1.0).unwrap();
        let dq = DeltaQ::cdf(cdf.clone());
        assert_eq!(dq.to_string(), format!("{:?}", cdf));
    }

    #[test]
    fn test_display_seq() {
        let dq1 = DeltaQ::name("A");
        let dq2 = DeltaQ::name("B");
        let seq = DeltaQ::seq(dq1, dq2);
        assert_eq!(seq.to_string(), "A •->-• B");
    }

    #[test]
    fn test_display_choice() {
        let dq1 = DeltaQ::name("A");
        let dq2 = DeltaQ::name("B");
        let choice = DeltaQ::choice(dq1, 0.3, dq2, 0.7);
        assert_eq!(choice.to_string(), "A 0.3⇌0.7 B");
    }

    #[test]
    fn test_display_for_all() {
        let dq1 = DeltaQ::name("A");
        let dq2 = DeltaQ::name("B");
        let for_all = DeltaQ::for_all(dq1, dq2);
        assert_eq!(for_all.to_string(), "∀(A | B)");
    }

    #[test]
    fn test_display_for_some() {
        let dq1 = DeltaQ::name("A");
        let dq2 = DeltaQ::name("B");
        let for_some = DeltaQ::for_some(dq1, dq2);
        assert_eq!(for_some.to_string(), "∃(A | B)");
    }

    #[test]
    fn test_display_nested_seq() {
        let dq1 = DeltaQ::name("A");
        let dq2 = DeltaQ::name("B");
        let dq3 = DeltaQ::name("C");
        let nested_seq = DeltaQ::seq(DeltaQ::seq(dq1, dq2), dq3);
        assert_eq!(nested_seq.to_string(), "(A •->-• B) •->-• C");
    }

    #[test]
    fn test_display_nested_choice() {
        let dq1 = DeltaQ::name("A");
        let dq2 = DeltaQ::name("B");
        let dq3 = DeltaQ::name("C");
        let nested_choice = DeltaQ::choice(DeltaQ::choice(dq1, 0.3, dq2, 0.7), 0.5, dq3, 0.5);
        assert_eq!(nested_choice.to_string(), "(A 0.3⇌0.7 B) 0.5⇌0.5 C");
    }

    #[test]
    fn test_display_nested_for_all() {
        let dq1 = DeltaQ::name("A");
        let dq2 = DeltaQ::name("B");
        let dq3 = DeltaQ::name("C");
        let dq4 = DeltaQ::name("D");
        let nested_for_all = DeltaQ::for_all(DeltaQ::for_all(dq1, dq2), DeltaQ::seq(dq3, dq4));
        assert_eq!(nested_for_all.to_string(), "∀(∀(A | B) | C •->-• D)");
    }

    #[test]
    fn test_display_nested_for_some() {
        let dq1 = DeltaQ::name("A");
        let dq2 = DeltaQ::name("B");
        let dq3 = DeltaQ::name("C");
        let dq4 = DeltaQ::name("D");
        let nested_for_some = DeltaQ::for_some(
            DeltaQ::for_some(dq1, dq2),
            DeltaQ::choice(dq3, 1.0, dq4, 2.0),
        );
        assert_eq!(nested_for_some.to_string(), "∃(∃(A | B) | C 1⇌2 D)");
    }
}
