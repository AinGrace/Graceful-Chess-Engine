#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Role {
    pub fn from_char(chr: char) -> Option<Self> {
        match chr {
            'p' | 'P' => Some(Self::Pawn),
            'n' | 'N' => Some(Self::Knight),
            'b' | 'B' => Some(Self::Bishop),
            'r' | 'R' => Some(Self::Rook),
            'q' | 'Q' => Some(Self::Queen),
            'k' | 'K' => Some(Self::King),
            _unknown => None,
        }
    }

    pub fn char(self) -> char {
        match self {
            Role::Pawn => 'p',
            Role::Knight => 'n',
            Role::Bishop => 'b',
            Role::Rook => 'r',
            Role::Queen => 'q',
            Role::King => 'k',
        }
    }
}

#[cfg(test)]
mod role_tests {
    use std::collections::HashSet;

    use super::*;

    const ALL_ROLES: [Role; 6] = [
        Role::Pawn,
        Role::Knight,
        Role::Bishop,
        Role::Rook,
        Role::Queen,
        Role::King,
    ];

    #[test]
    fn from_char_lowercase_all() {
        assert_eq!(Role::from_char('p'), Some(Role::Pawn));
        assert_eq!(Role::from_char('n'), Some(Role::Knight));
        assert_eq!(Role::from_char('b'), Some(Role::Bishop));
        assert_eq!(Role::from_char('r'), Some(Role::Rook));
        assert_eq!(Role::from_char('q'), Some(Role::Queen));
        assert_eq!(Role::from_char('k'), Some(Role::King));
    }

    #[test]
    fn from_char_uppercase_all() {
        assert_eq!(Role::from_char('P'), Some(Role::Pawn));
        assert_eq!(Role::from_char('N'), Some(Role::Knight));
        assert_eq!(Role::from_char('B'), Some(Role::Bishop));
        assert_eq!(Role::from_char('R'), Some(Role::Rook));
        assert_eq!(Role::from_char('Q'), Some(Role::Queen));
        assert_eq!(Role::from_char('K'), Some(Role::King));
    }

    #[test]
    fn from_char_upper_and_lower_produce_same_variant() {
        let pairs = [
            ('p', 'P'),
            ('n', 'N'),
            ('b', 'B'),
            ('r', 'R'),
            ('q', 'Q'),
            ('k', 'K'),
        ];
        for (lower, upper) in pairs {
            assert_eq!(
                Role::from_char(lower),
                Role::from_char(upper),
                "'{lower}' and '{upper}' should map to the same Role"
            );
        }
    }

    #[test]
    fn from_char_unrelated_letters_are_none() {
        for c in [
            'a', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'm', 'o', 's', 't', 'A', 'C', 'D', 'E',
            'F', 'G', 'H', 'I', 'J', 'M', 'O', 'S', 'T',
        ] {
            assert_eq!(Role::from_char(c), None, "'{c}' should return None");
        }
    }

    #[test]
    fn from_char_digits_are_none() {
        for c in '0'..='9' {
            assert_eq!(Role::from_char(c), None, "digit '{c}' should return None");
        }
    }

    #[test]
    fn from_char_special_chars_are_none() {
        for c in [' ', '\n', '\t', '-', '_', '!', '/', '.', ','] {
            assert_eq!(
                Role::from_char(c),
                None,
                "special char '{c}' should return None"
            );
        }
    }

    #[test]
    fn char_returns_correct_lowercase_letter() {
        assert_eq!(Role::Pawn.char(), 'p');
        assert_eq!(Role::Knight.char(), 'n');
        assert_eq!(Role::Bishop.char(), 'b');
        assert_eq!(Role::Rook.char(), 'r');
        assert_eq!(Role::Queen.char(), 'q');
        assert_eq!(Role::King.char(), 'k');
    }

    #[test]
    fn char_is_always_lowercase() {
        for role in ALL_ROLES {
            let c = role.char();
            assert!(
                c.is_lowercase(),
                "char() for {role:?} returned '{c}', expected lowercase"
            );
        }
    }

    #[test]
    fn char_is_always_alphabetic() {
        for role in ALL_ROLES {
            let c = role.char();
            assert!(
                c.is_alphabetic(),
                "char() for {role:?} returned '{c}', expected alphabetic"
            );
        }
    }

    #[test]
    fn char_values_are_unique() {
        let chars: Vec<char> = ALL_ROLES.iter().map(|r| r.char()).collect();
        let unique: HashSet<char> = chars.iter().copied().collect();
        assert_eq!(
            chars.len(),
            unique.len(),
            "all char() values should be distinct"
        );
    }

    #[test]
    fn char_then_new_round_trips() {
        for role in ALL_ROLES {
            let c = role.char();
            assert_eq!(
                Role::from_char(c),
                Some(role),
                "round-trip failed for {role:?}"
            );
        }
    }

    #[test]
    fn from_char_lowercase_then_char_is_identity() {
        for c in ['p', 'n', 'b', 'r', 'q', 'k'] {
            let role = Role::from_char(c).unwrap();
            assert_eq!(role.char(), c, "round-trip failed for '{c}'");
        }
    }

    #[test]
    fn from_char_uppercase_then_char_is_lowercase() {
        let pairs = [
            ('P', 'p'),
            ('N', 'n'),
            ('B', 'b'),
            ('R', 'r'),
            ('Q', 'q'),
            ('K', 'k'),
        ];
        for (upper, lower) in pairs {
            let role = Role::from_char(upper).unwrap();
            assert_eq!(
                role.char(),
                lower,
                "uppercase '{upper}' should round-trip to '{lower}'"
            );
        }
    }

    #[test]
    fn pawn_is_least_king_is_greatest() {
        assert!(Role::Pawn < Role::King);
    }

    #[test]
    fn ordering_is_sequential() {
        for window in ALL_ROLES.windows(2) {
            assert!(
                window[0] < window[1],
                "{:?} should be less than {:?}",
                window[0],
                window[1]
            );
        }
    }

    #[test]
    fn ordering_is_transitive() {
        // Spot-check a few non-adjacent pairs
        assert!(Role::Pawn < Role::Bishop);
        assert!(Role::Knight < Role::Rook);
        assert!(Role::Bishop < Role::Queen);
        assert!(Role::Rook < Role::King);
    }

    #[test]
    fn min_is_pawn_max_is_king() {
        assert_eq!(ALL_ROLES.iter().copied().min(), Some(Role::Pawn));
        assert_eq!(ALL_ROLES.iter().copied().max(), Some(Role::King));
    }

    #[test]
    fn sort_produces_declaration_order() {
        let mut shuffled = [
            Role::King,
            Role::Pawn,
            Role::Queen,
            Role::Bishop,
            Role::Rook,
            Role::Knight,
        ];
        shuffled.sort();
        assert_eq!(shuffled, ALL_ROLES);
    }
}
