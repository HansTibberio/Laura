/*
    Laura-Core: a fast and efficient move generator for chess engines.

    Copyright (C) 2024-2025 HansTibberio <hanstiberio@proton.me>

    Laura-Core is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Laura-Core is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Laura-Core. If not, see <https://www.gnu.org/licenses/>.
*/

use crate::{BitBoard, Square};

// Include precomputed table of between Bitboards for rooks, bishops and queens.
// These tables are generated during the build process and stored in
// the specified output directory.
include!(concat!(env!("OUT_DIR"), "/between_array.rs"));

/// Precomputed rays for bishops, indexed by square.
/// This constant holds the BitBoards representing the rays a bishop can attack from each square.
pub(crate) const BISHOP_RAYS: [BitBoard; Square::NUM_SQUARES] = [
    BitBoard(9241421688590303744),
    BitBoard(36099303471056128),
    BitBoard(141012904249856),
    BitBoard(550848566272),
    BitBoard(6480472064),
    BitBoard(1108177604608),
    BitBoard(283691315142656),
    BitBoard(72624976668147712),
    BitBoard(4620710844295151618),
    BitBoard(9241421688590368773),
    BitBoard(36099303487963146),
    BitBoard(141017232965652),
    BitBoard(1659000848424),
    BitBoard(283693466779728),
    BitBoard(72624976676520096),
    BitBoard(145249953336262720),
    BitBoard(2310355422147510788),
    BitBoard(4620710844311799048),
    BitBoard(9241421692918565393),
    BitBoard(36100411639206946),
    BitBoard(424704217196612),
    BitBoard(72625527495610504),
    BitBoard(145249955479592976),
    BitBoard(290499906664153120),
    BitBoard(1155177711057110024),
    BitBoard(2310355426409252880),
    BitBoard(4620711952330133792),
    BitBoard(9241705379636978241),
    BitBoard(108724279602332802),
    BitBoard(145390965166737412),
    BitBoard(290500455356698632),
    BitBoard(580999811184992272),
    BitBoard(577588851267340304),
    BitBoard(1155178802063085600),
    BitBoard(2310639079102947392),
    BitBoard(4693335752243822976),
    BitBoard(9386671504487645697),
    BitBoard(326598935265674242),
    BitBoard(581140276476643332),
    BitBoard(1161999073681608712),
    BitBoard(288793334762704928),
    BitBoard(577868148797087808),
    BitBoard(1227793891648880768),
    BitBoard(2455587783297826816),
    BitBoard(4911175566595588352),
    BitBoard(9822351133174399489),
    BitBoard(1197958188344280066),
    BitBoard(2323857683139004420),
    BitBoard(144117404414255168),
    BitBoard(360293502378066048),
    BitBoard(720587009051099136),
    BitBoard(1441174018118909952),
    BitBoard(2882348036221108224),
    BitBoard(5764696068147249408),
    BitBoard(11529391036782871041),
    BitBoard(4611756524879479810),
    BitBoard(567382630219904),
    BitBoard(1416240237150208),
    BitBoard(2833579985862656),
    BitBoard(5667164249915392),
    BitBoard(11334324221640704),
    BitBoard(22667548931719168),
    BitBoard(45053622886727936),
    BitBoard(18049651735527937),
];

/// Precomputed rays for rooks, indexed by square.
/// This constant holds the BitBoards representing the rays a rook can attack from each square.
pub(crate) const ROOK_RAYS: [BitBoard; Square::NUM_SQUARES] = [
    BitBoard(72340172838076926),
    BitBoard(144680345676153597),
    BitBoard(289360691352306939),
    BitBoard(578721382704613623),
    BitBoard(1157442765409226991),
    BitBoard(2314885530818453727),
    BitBoard(4629771061636907199),
    BitBoard(9259542123273814143),
    BitBoard(72340172838141441),
    BitBoard(144680345676217602),
    BitBoard(289360691352369924),
    BitBoard(578721382704674568),
    BitBoard(1157442765409283856),
    BitBoard(2314885530818502432),
    BitBoard(4629771061636939584),
    BitBoard(9259542123273813888),
    BitBoard(72340172854657281),
    BitBoard(144680345692602882),
    BitBoard(289360691368494084),
    BitBoard(578721382720276488),
    BitBoard(1157442765423841296),
    BitBoard(2314885530830970912),
    BitBoard(4629771061645230144),
    BitBoard(9259542123273748608),
    BitBoard(72340177082712321),
    BitBoard(144680349887234562),
    BitBoard(289360695496279044),
    BitBoard(578721386714368008),
    BitBoard(1157442769150545936),
    BitBoard(2314885534022901792),
    BitBoard(4629771063767613504),
    BitBoard(9259542123257036928),
    BitBoard(72341259464802561),
    BitBoard(144681423712944642),
    BitBoard(289361752209228804),
    BitBoard(578722409201797128),
    BitBoard(1157443723186933776),
    BitBoard(2314886351157207072),
    BitBoard(4629771607097753664),
    BitBoard(9259542118978846848),
    BitBoard(72618349279904001),
    BitBoard(144956323094725122),
    BitBoard(289632270724367364),
    BitBoard(578984165983651848),
    BitBoard(1157687956502220816),
    BitBoard(2315095537539358752),
    BitBoard(4629910699613634624),
    BitBoard(9259541023762186368),
    BitBoard(143553341945872641),
    BitBoard(215330564830528002),
    BitBoard(358885010599838724),
    BitBoard(645993902138460168),
    BitBoard(1220211685215703056),
    BitBoard(2368647251370188832),
    BitBoard(4665518383679160384),
    BitBoard(9259260648297103488),
    BitBoard(18302911464433844481),
    BitBoard(18231136449196065282),
    BitBoard(18087586418720506884),
    BitBoard(17800486357769390088),
    BitBoard(17226286235867156496),
    BitBoard(16077885992062689312),
    BitBoard(13781085504453754944),
    BitBoard(9187484529235886208),
];

/// Retrieves the BitBoard representing all the squares between the source and destination squares,
/// based on the precomputed between table for rooks, bishops, or queens.
#[inline]
pub fn get_between(src: Square, dest: Square) -> BitBoard {
    unsafe {
        BitBoard(
            *BETWEEN_ARRAY
                .get_unchecked(src as usize)
                .get_unchecked(dest as usize),
        )
    }
}

/// Retrieves the BitBoard representing the rays a bishop can attack from a given square.
#[inline(always)]
pub fn get_bishop_rays(square: Square) -> BitBoard {
    unsafe { *BISHOP_RAYS.get_unchecked(square.to_index()) }
}

/// Retrieves the BitBoard representing the rays a rook can attack from a given square.
#[inline(always)]
pub fn get_rook_rays(square: Square) -> BitBoard {
    unsafe { *ROOK_RAYS.get_unchecked(square.to_index()) }
}
