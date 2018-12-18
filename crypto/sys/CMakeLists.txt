cmake_minimum_required(VERSION 3.3.1)
project(cncrypto C)

include_directories(${CMAKE_SOURCE_DIR})

set (source_files
    aesb.c
    blake256.c
    crypto-ops-data.c
    crypto-ops.c
    groestl.c
    hash-extra-blake.c
    hash-extra-groestl.c
    hash-extra-jh.c
    hash-extra-skein.c
    hash.c
    jh.c
    keccak.c
    oaes_lib.c
    skein.c
    slow-hash.c
)

add_library(libcncrypto SHARED
    ${source_files}
)
target_include_directories(libcncrypto PUBLIC ${CMAKE_SOURCE_DIR})