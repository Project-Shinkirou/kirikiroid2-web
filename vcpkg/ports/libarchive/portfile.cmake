vcpkg_from_github(
    OUT_SOURCE_PATH SOURCE_PATH
    REPO libarchive/libarchive
    REF "v${VERSION}"
    SHA512 95c6232d178b26daa0eeba43d64ea4235aa96fa279c85fff715ac5e6cc73b2e65f276770f91c3538cb8ca989380555169497628d73e120bfa52e12f657049ff0
    HEAD_REF master
)

set(platform_options)
if(VCPKG_TARGET_IS_OSX OR VCPKG_TARGET_IS_IOS)
    set(apple_sdk "${VCPKG_OSX_SYSROOT}")
    if(NOT apple_sdk)
        if(VCPKG_TARGET_IS_IOS)
            if(VCPKG_TARGET_ARCHITECTURE STREQUAL "x64")
                set(apple_sdk iphonesimulator)
            else()
                set(apple_sdk iphoneos)
            endif()
        else()
            set(apple_sdk macosx)
        endif()
    endif()
    if(NOT IS_ABSOLUTE "${apple_sdk}")
        execute_process(
            COMMAND xcrun --sdk "${apple_sdk}" --show-sdk-path
            OUTPUT_VARIABLE apple_sdk
            OUTPUT_STRIP_TRAILING_WHITESPACE
            COMMAND_ERROR_IS_FATAL ANY
        )
    endif()
    # Libarchive adds /opt/local to its own search prefixes. Use matching SDK
    # iconv headers and libraries, never host MacPorts headers with SDK linkage.
    list(APPEND platform_options
        "-DICONV_INCLUDE_DIR=${apple_sdk}/usr/include"
        "-DLIBICONV_PATH=${apple_sdk}/usr/lib/libiconv.tbd"
        "-DLIBCHARSET_PATH=${apple_sdk}/usr/lib/libcharset.tbd"
    )
endif()

if(VCPKG_TARGET_IS_IOS)
    # The embedded core consumes libarchive, not standalone iOS command apps.
    # Keep every archive reader/writer and compression dependency in the library.
    list(APPEND platform_options
        -DENABLE_TAR=OFF
        -DENABLE_CPIO=OFF
        -DENABLE_CAT=OFF
        -DENABLE_UNZIP=OFF
    )
endif()

vcpkg_cmake_configure(
    SOURCE_PATH "${SOURCE_PATH}"
    OPTIONS
        -DENABLE_OPENSSL=OFF
        -DENABLE_TEST=OFF
        -DENABLE_LIBXML2=OFF
        -DENABLE_LIBB2=OFF
        ${platform_options}
)

vcpkg_cmake_install()

vcpkg_fixup_pkgconfig()

vcpkg_copy_pdbs()

file(REMOVE_RECURSE
      "${CURRENT_PACKAGES_DIR}/debug/include"
      "${CURRENT_PACKAGES_DIR}/debug/share"
      "${CURRENT_PACKAGES_DIR}/share/man"
)

foreach(header "include/archive.h" "include/archive_entry.h")
    vcpkg_replace_string("${CURRENT_PACKAGES_DIR}/${header}" "(!defined LIBARCHIVE_STATIC)" "0")
endforeach()

file(INSTALL "${CURRENT_PORT_DIR}/usage" DESTINATION "${CURRENT_PACKAGES_DIR}/share/${PORT}")
file(INSTALL "${SOURCE_PATH}/COPYING" DESTINATION "${CURRENT_PACKAGES_DIR}/share/${PORT}" RENAME copyright)
