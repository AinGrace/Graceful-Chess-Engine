use std::hint::unreachable_unchecked;

use crate::{bitboard::Bitboard, role::Role, square::Square};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ByRole<T> {
    inner: [T; 6],
}

impl<T> ByRole<T> {
    pub fn new(pawn: T, knight: T, bishop: T, rook: T, queen: T, king: T) -> Self {
        Self {
            inner: [pawn, knight, bishop, rook, queen, king],
        }
    }

    pub fn get(&self, role: Role) -> &T {
        unsafe { self.inner.get_unchecked(role as usize) }
    }

    pub fn get_mut(&mut self, role: Role) -> &mut T {
        unsafe { self.inner.get_unchecked_mut(role as usize) }
    }

    pub fn pawns(&self) -> &T {
        self.get(Role::Pawn)
    }

    pub fn pawns_mut(&mut self) -> &mut T {
        self.get_mut(Role::Pawn)
    }

    pub fn knights(&self) -> &T {
        self.get(Role::Knight)
    }

    pub fn knights_mut(&mut self) -> &mut T {
        self.get_mut(Role::Knight)
    }

    pub fn bishops(&self) -> &T {
        self.get(Role::Bishop)
    }

    pub fn bishops_mut(&mut self) -> &mut T {
        self.get_mut(Role::Bishop)
    }

    pub fn rooks(&self) -> &T {
        self.get(Role::Rook)
    }

    pub fn rooks_mut(&mut self) -> &mut T {
        self.get_mut(Role::Rook)
    }

    pub fn queens(&self) -> &T {
        self.get(Role::Queen)
    }
    pub fn queens_mut(&mut self) -> &mut T {
        self.get_mut(Role::Queen)
    }

    pub fn kings(&self) -> &T {
        self.get(Role::King)
    }

    pub fn kings_mut(&mut self) -> &mut T {
        self.get_mut(Role::King)
    }
}

impl ByRole<Bitboard> {
    pub fn peek_role(&self, square: Square) -> Option<Role> {
        if self.pawns().is_square_set(square) {
            return Some(Role::Pawn);
        }

        if self.knights().is_square_set(square) {
            return Some(Role::Knight);
        }

        if self.bishops().is_square_set(square) {
            return Some(Role::Bishop);
        }

        if self.rooks().is_square_set(square) {
            return Some(Role::Rook);
        }

        if self.queens().is_square_set(square) {
            return Some(Role::Queen);
        }

        if self.kings().is_square_set(square) {
            return Some(Role::King);
        }

        None
    }

    pub fn peek_role_checked(&self, square: Square) -> Role {
        if self.pawns().is_square_set(square) {
            return Role::Pawn;
        }

        if self.knights().is_square_set(square) {
            return Role::Knight;
        }

        if self.bishops().is_square_set(square) {
            return Role::Bishop;
        }

        if self.rooks().is_square_set(square) {
            return Role::Rook;
        }

        if self.queens().is_square_set(square) {
            return Role::Queen;
        }

        if self.kings().is_square_set(square) {
            return Role::King;
        }

        unreachable!("peek_role_checked on empty square {square}")
    }

    /// caller should guarantee that Square does correspont to existing piece on bitboard
    pub unsafe fn peek_role_unchecked(&self, square: Square) -> Role {
        if self.pawns().is_square_set(square) {
            return Role::Pawn;
        }

        if self.knights().is_square_set(square) {
            return Role::Knight;
        }

        if self.bishops().is_square_set(square) {
            return Role::Bishop;
        }

        if self.rooks().is_square_set(square) {
            return Role::Rook;
        }

        if self.queens().is_square_set(square) {
            return Role::Queen;
        }

        if self.kings().is_square_set(square) {
            return Role::King;
        }

        unsafe { unreachable_unchecked() }
    }
}

#[cfg(test)]
mod by_role_tests {

    use crate::{
        bitboard::{Bitboard, ToBitboard},
        role::Role,
        square::Square,
    };

    type ByRole = super::ByRole<Bitboard>;

    const ALL_ROLES: [Role; 6] = [
        Role::Pawn,
        Role::Knight,
        Role::Bishop,
        Role::Rook,
        Role::Queen,
        Role::King,
    ];

    // Six squares, one per role — all distinct, spread across the board
    const ROLE_SQUARES: [(Role, Square); 6] = [
        (Role::Pawn, Square::A2),
        (Role::Knight, Square::B1),
        (Role::Bishop, Square::C1),
        (Role::Rook, Square::A1),
        (Role::Queen, Square::D1),
        (Role::King, Square::E1),
    ];

    /// each role board contains exactly one distinct square.
    fn fixture() -> ByRole {
        ByRole::new(
            Square::A2.to_bb(),
            Square::B1.to_bb(),
            Square::C1.to_bb(),
            Square::A1.to_bb(),
            Square::D1.to_bb(),
            Square::E1.to_bb(),
        )
    }

    /// all-empty ByRole.
    fn empty() -> ByRole {
        ByRole::new(
            Bitboard::new_empty(),
            Bitboard::new_empty(),
            Bitboard::new_empty(),
            Bitboard::new_empty(),
            Bitboard::new_empty(),
            Bitboard::new_empty(),
        )
    }

    #[test]
    fn new_stores_each_bitboard_independently() {
        let by_role = fixture();
        assert_eq!(by_role.pawns().as_u64(), Square::A2.to_bb().as_u64());
        assert_eq!(by_role.knights().as_u64(), Square::B1.to_bb().as_u64());
        assert_eq!(by_role.bishops().as_u64(), Square::C1.to_bb().as_u64());
        assert_eq!(by_role.rooks().as_u64(), Square::A1.to_bb().as_u64());
        assert_eq!(by_role.queens().as_u64(), Square::D1.to_bb().as_u64());
        assert_eq!(by_role.kings().as_u64(), Square::E1.to_bb().as_u64());
    }

    #[test]
    fn new_empty_all_zero() {
        let by_role = empty();
        for role in ALL_ROLES {
            assert!(
                by_role.get(role).empty(),
                "{role:?} bitboard should be empty"
            );
        }
    }

    #[test]
    fn get_returns_correct_bitboard_for_each_role() {
        let by_role = fixture();
        for (role, square) in ROLE_SQUARES {
            let bb = by_role.get(role);
            assert!(
                bb.is_square_set(square),
                "get({role:?}) should contain {square:?}"
            );
        }
    }

    #[test]
    fn get_matches_named_accessors() {
        let by_role = fixture();
        assert_eq!(by_role.get(Role::Pawn).as_u64(), by_role.pawns().as_u64());
        assert_eq!(
            by_role.get(Role::Knight).as_u64(),
            by_role.knights().as_u64()
        );
        assert_eq!(
            by_role.get(Role::Bishop).as_u64(),
            by_role.bishops().as_u64()
        );
        assert_eq!(by_role.get(Role::Rook).as_u64(), by_role.rooks().as_u64());
        assert_eq!(by_role.get(Role::Queen).as_u64(), by_role.queens().as_u64());
        assert_eq!(by_role.get(Role::King).as_u64(), by_role.kings().as_u64());
    }

    #[test]
    fn get_does_not_alias_other_roles() {
        let by_role = fixture();
        for (role, square) in ROLE_SQUARES {
            for (other_role, _) in ROLE_SQUARES {
                if role == other_role {
                    continue;
                }
                assert!(
                    !by_role.get(other_role).is_square_set(square),
                    "get({other_role:?}) should not contain {square:?} which belongs to {role:?}"
                );
            }
        }
    }

    #[test]
    fn get_mut_modifies_correct_field() {
        for target_role in ALL_ROLES {
            let mut by_role = empty();
            let new_sq = Square::H8;
            *by_role.get_mut(target_role) = new_sq.to_bb();

            assert!(by_role.get(target_role).is_square_set(new_sq));
            // All other roles must remain empty
            for other_role in ALL_ROLES {
                if other_role != target_role {
                    assert!(
                        by_role.get(other_role).empty(),
                        "mutating {target_role:?} should not affect {other_role:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn get_mut_matches_named_mut_accessors() {
        let mut by_role1 = empty();
        let sq = Square::G7.to_bb();

        *by_role1.get_mut(Role::Pawn) = sq;
        *by_role1.get_mut(Role::Knight) = sq;
        *by_role1.get_mut(Role::Bishop) = sq;
        *by_role1.get_mut(Role::Rook) = sq;
        *by_role1.get_mut(Role::Queen) = sq;
        *by_role1.get_mut(Role::King) = sq;

        // Compare via named accessors instead
        assert_eq!(by_role1.get(Role::Pawn).as_u64(), sq.as_u64());
        assert_eq!(by_role1.get(Role::Knight).as_u64(), sq.as_u64());
        assert_eq!(by_role1.get(Role::Bishop).as_u64(), sq.as_u64());
        assert_eq!(by_role1.get(Role::Rook).as_u64(), sq.as_u64());
        assert_eq!(by_role1.get(Role::Queen).as_u64(), sq.as_u64());
        assert_eq!(by_role1.get(Role::King).as_u64(), sq.as_u64());
    }

    #[test]
    fn named_mut_accessors_modify_only_their_field() {
        let mutations: &[(fn(&mut ByRole) -> &mut Bitboard, Role)] = &[
            (|b| b.pawns_mut(), Role::Pawn),
            (|b| b.knights_mut(), Role::Knight),
            (|b| b.bishops_mut(), Role::Bishop),
            (|b| b.rooks_mut(), Role::Rook),
            (|b| b.queens_mut(), Role::Queen),
            (|b| b.kings_mut(), Role::King),
        ];

        for (accessor, target) in mutations {
            let mut by_role = empty();
            *accessor(&mut by_role) = Square::H8.to_bb();

            assert!(by_role.get(*target).is_square_set(Square::H8));
            for other in ALL_ROLES {
                if other != *target {
                    assert!(by_role.get(other).empty());
                }
            }
        }
    }

    #[test]
    fn peek_role_returns_correct_role_for_each_square() {
        let by_role = fixture();
        for (role, square) in ROLE_SQUARES {
            assert_eq!(
                by_role.peek_role(square),
                Some(role),
                "peek_role({square:?}) should return {role:?}"
            );
        }
    }

    #[test]
    fn peek_role_returns_none_on_empty_board() {
        let by_role = empty();
        for i in 0u32..64 {
            let sq = Square::from_u32_checked(i);
            assert_eq!(
                by_role.peek_role(sq),
                None,
                "empty board should return None for {sq:?}"
            );
        }
    }

    #[test]
    fn peek_role_returns_none_for_unoccupied_square() {
        let by_role = fixture();
        // H8 is not used by any role in the fixture
        assert_eq!(by_role.peek_role(Square::H8), None);
    }

    #[test]
    fn peek_role_priority_pawn_before_knight() {
        // If a square is set in two fields, peek_role returns the first match (Pawn wins)
        let shared_sq = Square::G5;
        let by_role = ByRole::new(
            shared_sq.to_bb(),
            shared_sq.to_bb(),
            Bitboard::new_empty(),
            Bitboard::new_empty(),
            Bitboard::new_empty(),
            Bitboard::new_empty(),
        );
        assert_eq!(by_role.peek_role(shared_sq), Some(Role::Pawn));
    }

    #[test]
    fn peek_role_all_64_squares_when_fully_occupied() {
        // Place all 64 squares in a single role and verify peek_role returns that role everywhere
        for target_role in ALL_ROLES {
            let mut bbs = [Bitboard::new_empty(); 6];
            bbs[target_role as usize] = Bitboard::from_u64(u64::MAX);
            let by_role = ByRole::new(bbs[0], bbs[1], bbs[2], bbs[3], bbs[4], bbs[5]);
            for i in 0u32..64 {
                let sq = Square::from_u32_checked(i);
                assert_eq!(by_role.peek_role(sq), Some(target_role));
            }
        }
    }

    #[test]
    fn peek_role_checked_returns_correct_role() {
        let by_role = fixture();
        for (role, square) in ROLE_SQUARES {
            assert_eq!(
                by_role.peek_role_checked(square),
                role,
                "peek_role_checked({square:?}) should return {role:?}"
            );
        }
    }

    #[test]
    fn peek_role_checked_matches_peek_role_when_some() {
        let by_role = fixture();
        for (_, square) in ROLE_SQUARES {
            assert_eq!(
                Some(by_role.peek_role_checked(square)),
                by_role.peek_role(square)
            );
        }
    }

    #[test]
    #[should_panic]
    fn peek_role_checked_panics_on_empty_square() {
        let by_role = empty();
        let _ = by_role.peek_role_checked(Square::H8);
    }

    #[test]
    #[should_panic]
    fn peek_role_checked_panics_on_unoccupied_square_in_fixture() {
        let by_role = fixture(); // H8 is never set
        let _ = by_role.peek_role_checked(Square::H8);
    }

    #[test]
    fn copy_is_independent() {
        let original = fixture();
        let mut copy = original;
        *copy.pawns_mut() = Bitboard::new_empty();

        // Original pawn board must be unchanged
        assert!(original.pawns().is_square_set(Square::A2));
        assert!(copy.pawns().empty());
    }

    #[test]
    fn clone_is_independent() {
        let original = fixture();
        let mut cloned = original.clone();
        *cloned.kings_mut() = Bitboard::new_empty();

        assert!(original.kings().is_square_set(Square::E1));
        assert!(cloned.kings().empty());
    }

    #[test]
    fn get_after_get_mut_reflects_change() {
        let mut by_role = fixture();
        for role in ALL_ROLES {
            let new_bb = Square::H7.to_bb();
            *by_role.get_mut(role) = new_bb;
            assert_eq!(by_role.get(role).as_u64(), new_bb.as_u64());
        }
    }

    #[test]
    fn get_mut_set_and_clear_square() {
        let mut by_role = empty();
        let bb = by_role.get_mut(Role::Rook);
        *bb = bb.set_square(Square::D4);
        assert!(by_role.get(Role::Rook).is_square_set(Square::D4));

        let bb = by_role.get_mut(Role::Rook);
        *bb = bb.clear_square(Square::D4);
        assert!(by_role.get(Role::Rook).empty());
    }
}
