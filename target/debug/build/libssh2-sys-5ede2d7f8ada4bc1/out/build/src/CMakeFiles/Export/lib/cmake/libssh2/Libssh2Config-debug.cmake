#----------------------------------------------------------------
# Generated CMake target import file for configuration "Debug".
#----------------------------------------------------------------

# Commands may need to know the format version.
set(CMAKE_IMPORT_FILE_VERSION 1)

# Import target "Libssh2::libssh2" for configuration "Debug"
set_property(TARGET Libssh2::libssh2 APPEND PROPERTY IMPORTED_CONFIGURATIONS DEBUG)
set_target_properties(Libssh2::libssh2 PROPERTIES
  IMPORTED_LINK_INTERFACE_LANGUAGES_DEBUG "C"
  IMPORTED_LINK_INTERFACE_LIBRARIES_DEBUG "/opt/chefdk/embedded/lib/libssl.dylib;/opt/chefdk/embedded/lib/libcrypto.dylib;/opt/chefdk/embedded/lib/libz.dylib"
  IMPORTED_LOCATION_DEBUG "${_IMPORT_PREFIX}/lib/libssh2.a"
  )

list(APPEND _IMPORT_CHECK_TARGETS Libssh2::libssh2 )
list(APPEND _IMPORT_CHECK_FILES_FOR_Libssh2::libssh2 "${_IMPORT_PREFIX}/lib/libssh2.a" )

# Commands beyond this point should not need to know the version.
set(CMAKE_IMPORT_FILE_VERSION)
