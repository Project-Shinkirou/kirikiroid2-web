#include <catch2/catch_test_macros.hpp>

#include "NarrowText.h"

TEST_CASE("Manager text detection is independent for each BOM-less script") {
    tTVPMbToWc defaultConverter = nullptr;
    std::u16string decoded;
    REQUIRE(TVPDecodeNarrowText({0xe3, 0x81, 0x82, 0x20}, defaultConverter,
                                false, decoded));
    CHECK(decoded == u"\u3042 ");
    REQUIRE(TVPDecodeNarrowText({0x82, 0xa0, 0xfa, 0x40}, defaultConverter,
                                false, decoded));
    CHECK(decoded == u"\u3042\u2170");
}

TEST_CASE("An explicit UTF-8 override rejects a CP932 script") {
    tTVPMbToWc utf8 = TVPUtf8MbToWc;
    std::u16string decoded;
    CHECK_FALSE(TVPDecodeNarrowText({0x82, 0xa0}, utf8, false, decoded));
}
