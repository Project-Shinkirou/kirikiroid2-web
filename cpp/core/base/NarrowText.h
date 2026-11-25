#pragma once

#include <cstdint>
#include <string>
#include <vector>

extern "C" int gbk_mbtowc(unsigned short *wc, const unsigned char *s);
extern "C" int sjis_mbtowc(unsigned short *wc, const unsigned char *s);

using tTVPMbToWc = int (*)(unsigned short *, const unsigned char *);

// The legacy narrow-text fallback handles BMP UTF-8; BOM-marked UTF-8 is
// decoded separately by the text stream.
inline int TVPUtf8MbToWc(unsigned short *wc, const unsigned char *s) {
    const unsigned int first = s[0];
    if(first < 0x80) {
        *wc = static_cast<unsigned short>(first);
        return 1;
    }
    if(first < 0xC2)
        return -1;
    if(first <= 0xDF) {
        const unsigned int second = s[1] ^ 0x80;
        if(second > 0x3F)
            return -1;
        *wc = static_cast<unsigned short>(((first & 0x1F) << 6) | second);
        return 2;
    }
    if(first > 0xEF || static_cast<signed char>(s[1]) > -65 ||
       static_cast<signed char>(s[2]) > -65)
        return -1;

    const unsigned int second = s[1];
    if(first == 0xE0 && second < 0xA0)
        return -1;

    *wc = static_cast<unsigned short>(((second ^ 0x80) << 6) |
                                     (first << 12) | (s[2] ^ 0x80));
    return 3;
}

inline bool TVPDecodeNarrowText(std::vector<std::uint8_t> raw,
                                tTVPMbToWc &defaultConverter,
                                bool rememberDetection,
                                std::u16string &decoded) {
    const auto size = raw.size();
    raw.push_back(0);
    decoded.clear();
    decoded.reserve(size);
    const auto decode = [&](tTVPMbToWc converter) {
        const unsigned char *current = raw.data();
        const unsigned char *end = current + size;
        while(current < end && *current) {
            unsigned short wc;
            const int consumed = converter(&wc, current);
            if(consumed < 1 || consumed > end - current) {
                decoded.clear();
                return false;
            }
            decoded.push_back(static_cast<char16_t>(wc));
            current += consumed;
        }
        return true;
    };

    // An explicitly selected converter always takes precedence.
    if(defaultConverter)
        return decode(defaultConverter);
    if(decode(sjis_mbtowc))
        return true;
    for(tTVPMbToWc converter : {TVPUtf8MbToWc, gbk_mbtowc}) {
        if(decode(converter)) {
            if(rememberDetection)
                defaultConverter = converter;
            return true;
        }
    }
    return false;
}
