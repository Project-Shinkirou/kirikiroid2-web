vcpkg_check_linkage(ONLY_STATIC_LIBRARY)

# Match the LLVM version in this fork's builtin registry baseline. Build only
# its standalone CPU runtime, not LLVM, Clang or an offload toolchain.
vcpkg_download_distfile(OPENMP_ARCHIVE
    URLS "https://github.com/llvm/llvm-project/releases/download/llvmorg-${VERSION}/openmp-${VERSION}.src.tar.xz"
    FILENAME "openmp-${VERSION}.src.tar.xz"
    SHA512 f96f5fd4c508f1390e53b943237aa7e1db1301ef660f0864305556d581275576d585ef222a82d2359d43ad8ed166096d9ec6c05ab0ee57a01679cff6b4ecba4b
)
vcpkg_download_distfile(CMAKE_ARCHIVE
    URLS "https://github.com/llvm/llvm-project/releases/download/llvmorg-${VERSION}/cmake-${VERSION}.src.tar.xz"
    FILENAME "cmake-${VERSION}.src.tar.xz"
    SHA512 1334647f4be280b41858aa272bebc65e935cab772001032f77040396ba7472fbd5eb6a1a0c042ab7156540075705b7f05c8de2f02e2ce9d7ec1ec27be6bef86f
)
vcpkg_extract_source_archive(SOURCE_PATH ARCHIVE "${OPENMP_ARCHIVE}")
vcpkg_extract_source_archive(CMAKE_SOURCE_PATH ARCHIVE "${CMAKE_ARCHIVE}")
# This is the documented sibling layout for LLVM's standalone source releases.
file(COPY "${CMAKE_SOURCE_PATH}/" DESTINATION "${SOURCE_PATH}/../cmake")

vcpkg_cmake_configure(
    SOURCE_PATH "${SOURCE_PATH}"
    OPTIONS
        -DLIBOMP_ENABLE_SHARED=OFF
        -DLIBOMP_INSTALL_ALIASES=OFF
)
vcpkg_cmake_install()
file(REMOVE_RECURSE "${CURRENT_PACKAGES_DIR}/debug/include")
vcpkg_install_copyright(FILE_LIST "${SOURCE_PATH}/LICENSE.TXT")
