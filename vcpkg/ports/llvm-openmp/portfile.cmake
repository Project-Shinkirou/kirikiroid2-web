vcpkg_check_linkage(ONLY_STATIC_LIBRARY)

# Xcode 16.4 emits the dynamic-dispatch ABI introduced in LLVM 19.
# Build only its standalone CPU runtime, not LLVM, Clang or offload.
vcpkg_download_distfile(OPENMP_ARCHIVE
    URLS "https://github.com/llvm/llvm-project/releases/download/llvmorg-${VERSION}/openmp-${VERSION}.src.tar.xz"
    FILENAME "openmp-${VERSION}.src.tar.xz"
    SHA512 a6ee7e672d6f175bab672fd7cc03914485f528fc6ff82a6d8e10192de6480263bbc99dbff373d68f68a16edf0a7b46982d0fc4198c2069ff807bb77464fece7a
)
vcpkg_download_distfile(CMAKE_ARCHIVE
    URLS "https://github.com/llvm/llvm-project/releases/download/llvmorg-${VERSION}/cmake-${VERSION}.src.tar.xz"
    FILENAME "cmake-${VERSION}.src.tar.xz"
    SHA512 3ff56e1a61dbace35d8c168ad5c94c5a09d92eb683494bfe1deabcb3640cf79b7422d44903d049d2298d54c2874f9a60c4f13f6795a3949e7eaefd5d42e62621
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
        -DOPENMP_ENABLE_LIBOMPTARGET=OFF
        -DOPENMP_ENABLE_OMPT_TOOLS=OFF
        -DLIBOMP_OMPD_SUPPORT=OFF
        -DLIBOMP_OMPD_GDB_SUPPORT=OFF
)
vcpkg_cmake_install()
file(REMOVE_RECURSE "${CURRENT_PACKAGES_DIR}/debug/include")
vcpkg_install_copyright(FILE_LIST "${SOURCE_PATH}/LICENSE.TXT")
