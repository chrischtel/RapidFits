#----------------------------------------------------------------
# Generated CMake target import file for configuration "Debug".
#----------------------------------------------------------------

# Commands may need to know the format version.
set(CMAKE_IMPORT_FILE_VERSION 1)

# Import target "cfitsio" for configuration "Debug"
set_property(TARGET cfitsio APPEND PROPERTY IMPORTED_CONFIGURATIONS DEBUG)
set_target_properties(cfitsio PROPERTIES
  IMPORTED_IMPLIB_DEBUG "${_IMPORT_PREFIX}/debug/lib/cfitsio.lib"
  IMPORTED_LOCATION_DEBUG "${_IMPORT_PREFIX}/debug/bin/cfitsio.dll"
  )

list(APPEND _cmake_import_check_targets cfitsio )
list(APPEND _cmake_import_check_files_for_cfitsio "${_IMPORT_PREFIX}/debug/lib/cfitsio.lib" "${_IMPORT_PREFIX}/debug/bin/cfitsio.dll" )

# Commands beyond this point should not need to know the version.
set(CMAKE_IMPORT_FILE_VERSION)
