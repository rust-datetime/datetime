extern crate datetime;
pub use datetime::Duration;


mod addition {
    use super::*;

    #[test]
    fn simple() {
        assert_eq!(Duration::of(10), Duration::of(2) + Duration::of(8))
    }

    #[test]
    fn milliseconds() {
        assert_eq!(Duration::of_ms(0, 500), Duration::of_ms(0, 167) + Duration::of_ms(0, 333))
    }

    #[test]
    fn wrapping() {
        assert_eq!(Duration::of_ms(1, 500), Duration::of_ms(0, 750) + Duration::of_ms(0, 750))
    }

    #[test]
    fn wrapping_exact() {
        assert_eq!(Duration::of(1), Duration::of_ms(0, 500) + Duration::of_ms(0, 500))
    }
}


mod subtraction {
    use super::*;

    #[test]
    fn simple() {
        assert_eq!(Duration::of(13), Duration::of(28) - Duration::of(15))
    }

    #[test]
    fn milliseconds() {
        assert_eq!(Duration::of_ms(0, 300), Duration::of_ms(0, 950) - Duration::of_ms(0, 650))
    }

    #[test]
    fn wrapping() {
        assert_eq!(Duration::of_ms(0, 750), Duration::of_ms(1, 500) - Duration::of_ms(0, 750))
    }

    #[test]
    fn wrapping_exact() {
        assert_eq!(Duration::of(1), Duration::of_ms(1, 500) - Duration::of_ms(0, 500))
    }
}


mod multiplication {
    use super::*;

    #[test]
    fn simple() {
        assert_eq!(Duration::of(16), Duration::of(8) * 2)
    }

    #[test]
    fn milliseconds() {
        assert_eq!(Duration::of(1), Duration::of_ms(0, 500) * 2)
    }
}
