# Install script for directory: /Users/chrisjones/.cargo/registry/src/github.com-1ecc6299db9ec823/libssh2-sys-0.1.41/libssh2/src

# Set the install prefix
if(NOT DEFINED CMAKE_INSTALL_PREFIX)
  set(CMAKE_INSTALL_PREFIX "/Users/chrisjones/projects/lambdastack/http2hpack/target/debug/build/libssh2-sys-5ede2d7f8ada4bc1/out")
endif()
string(REGEX REPLACE "/$" "" CMAKE_INSTALL_PREFIX "${CMAKE_INSTALL_PREFIX}")

# Set the install configuration name.
if(NOT DEFINED CMAKE_INSTALL_CONFIG_NAME)
  if(BUILD_TYPE)
    string(REGEX REPLACE "^[^A-Za-z0-9_]+" ""
           CMAKE_INSTALL_CONFIG_NAME "${BUILD_TYPE}")
  else()
    set(CMAKE_INSTALL_CONFIG_NAME "Debug")
  endif()
  message(STATUS "Install configuration: \"${CMAKE_INSTALL_CONFIG_NAME}\"")
endif()

# Set the component getting installed.
if(NOT CMAKE_INSTALL_COMPONENT)
  if(COMPONENT)
    message(STATUS "Install component: \"${COMPONENT}\"")
    set(CMAKE_INSTALL_COMPONENT "${COMPONENT}")
  else()
    set(CMAKE_INSTALL_COMPONENT)
  endif()
endif()

if(NOT CMAKE_INSTALL_COMPONENT OR "${CMAKE_INSTALL_COMPONENT}" STREQUAL "Unspecified")
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/include" TYPE FILE FILES
    "/Users/chrisjones/.cargo/registry/src/github.com-1ecc6299db9ec823/libssh2-sys-0.1.41/libssh2/include/libssh2.h"
    "/Users/chrisjones/.cargo/registry/src/github.com-1ecc6299db9ec823/libssh2-sys-0.1.41/libssh2/include/libssh2_publickey.h"
    "/Users/chrisjones/.cargo/registry/src/github.com-1ecc6299db9ec823/libssh2-sys-0.1.41/libssh2/include/libssh2_sftp.h"
    )
endif()

if(NOT CMAKE_INSTALL_COMPONENT OR "${CMAKE_INSTALL_COMPONENT}" STREQUAL "Unspecified")
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib" TYPE STATIC_LIBRARY FILES "/Users/chrisjones/projects/lambdastack/http2hpack/target/debug/build/libssh2-sys-5ede2d7f8ada4bc1/out/build/src/libssh2.a")
  if(EXISTS "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib/libssh2.a" AND
     NOT IS_SYMLINK "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib/libssh2.a")
    execute_process(COMMAND "/usr/bin/ranlib" "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib/libssh2.a")
  endif()
endif()

if(NOT CMAKE_INSTALL_COMPONENT OR "${CMAKE_INSTALL_COMPONENT}" STREQUAL "Unspecified")
  if(EXISTS "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib/cmake/libssh2/Libssh2Config.cmake")
    file(DIFFERENT EXPORT_FILE_CHANGED FILES
         "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib/cmake/libssh2/Libssh2Config.cmake"
         "/Users/chrisjones/projects/lambdastack/http2hpack/target/debug/build/libssh2-sys-5ede2d7f8ada4bc1/out/build/src/CMakeFiles/Export/lib/cmake/libssh2/Libssh2Config.cmake")
    if(EXPORT_FILE_CHANGED)
      file(GLOB OLD_CONFIG_FILES "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib/cmake/libssh2/Libssh2Config-*.cmake")
      if(OLD_CONFIG_FILES)
        message(STATUS "Old export file \"$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib/cmake/libssh2/Libssh2Config.cmake\" will be replaced.  Removing files [${OLD_CONFIG_FILES}].")
        file(REMOVE ${OLD_CONFIG_FILES})
      endif()
    endif()
  endif()
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib/cmake/libssh2" TYPE FILE FILES "/Users/chrisjones/projects/lambdastack/http2hpack/target/debug/build/libssh2-sys-5ede2d7f8ada4bc1/out/build/src/CMakeFiles/Export/lib/cmake/libssh2/Libssh2Config.cmake")
  if("${CMAKE_INSTALL_CONFIG_NAME}" MATCHES "^([Dd][Ee][Bb][Uu][Gg])$")
    file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib/cmake/libssh2" TYPE FILE FILES "/Users/chrisjones/projects/lambdastack/http2hpack/target/debug/build/libssh2-sys-5ede2d7f8ada4bc1/out/build/src/CMakeFiles/Export/lib/cmake/libssh2/Libssh2Config-debug.cmake")
  endif()
endif()

if(NOT CMAKE_INSTALL_COMPONENT OR "${CMAKE_INSTALL_COMPONENT}" STREQUAL "Unspecified")
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib/pkgconfig" TYPE FILE FILES "/Users/chrisjones/projects/lambdastack/http2hpack/target/debug/build/libssh2-sys-5ede2d7f8ada4bc1/out/build/src/libssh2.pc")
endif()

if(NOT CMAKE_INSTALL_COMPONENT OR "${CMAKE_INSTALL_COMPONENT}" STREQUAL "Unspecified")
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib/cmake/libssh2" TYPE FILE FILES "/Users/chrisjones/projects/lambdastack/http2hpack/target/debug/build/libssh2-sys-5ede2d7f8ada4bc1/out/build/src/Libssh2ConfigVersion.cmake")
endif()

