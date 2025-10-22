#----------------------------------------------------------------
# Generated CMake target import file for configuration "Release".
#----------------------------------------------------------------

# Commands may need to know the format version.
set(CMAKE_IMPORT_FILE_VERSION 1)

# Import target "cfitsio" for configuration "Release"
set_property(TARGET cfitsio APPEND PROPERTY IMPORTED_CONFIGURATIONS RELEASE)
set_target_properties(cfitsio PROPERTIES
  IMPORTED_IMPLIB_RELEASE "${_IMPORT_PREFIX}/lib/cfitsio.lib"
  IMPORTED_LOCATION_RELEASE "${_IMPORT_PREFIX}/bin/cfitsio.dll"
  )

list(APPEND _cmake_import_check_targets cfitsio )
list(APPEND _cmake_import_check_files_for_cfitsio "${_IMPORT_PREFIX}/lib/cfitsio.lib" "${_IMPORT_PREFIX}/bin/cfitsio.dll" )

# Commands beyond this point should not need to know the version.
set(CMAKE_IMPORT_FILE_VERSION)
