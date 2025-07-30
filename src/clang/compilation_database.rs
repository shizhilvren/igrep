use std::mem;
use std::path::{Path, PathBuf};
#[macro_use]
use crate::clang::utility;
use crate::clang::utility::Nullable;
use clang_sys::*;

pub struct CompilationDatabase {
    ptr: clang_sys::CXCompilationDatabase,
}

impl CompilationDatabase {
    /// Creates a compilation database from the database found in the given directory.
    pub fn from_directory<P: AsRef<Path>>(path: P) -> Result<CompilationDatabase, ()> {
        let path = utility::from_path(path);
        unsafe {
            let mut error = mem::MaybeUninit::uninit();
            let ptr = clang_sys::clang_CompilationDatabase_fromDirectory(
                path.as_ptr(),
                error.as_mut_ptr(),
            );
            match error.assume_init() {
                clang_sys::CXCompilationDatabase_NoError => Ok(CompilationDatabase { ptr }),
                clang_sys::CXCompilationDatabase_CanNotLoadDatabase => Err(()),
                _ => unreachable!(),
            }
        }
    }

    /// Get all the compile commands from the database.
    pub fn get_all_compile_commands(&self) -> CompileCommands {
        unsafe {
            CompileCommands::from_ptr(clang_sys::clang_CompilationDatabase_getAllCompileCommands(
                self.ptr,
            ))
        }
    }

    /// Find the compile commands for the given file.
    pub fn get_compile_commands<P: AsRef<Path>>(&self, path: P) -> Result<CompileCommands, ()> {
        // Presumably this returns null if we can't find the given path?
        // The Clang docs don't specify.
        let path = utility::from_path(path);
        let ptr = unsafe { clang_CompilationDatabase_getCompileCommands(self.ptr, path.as_ptr()) };
        ptr.map(CompileCommands::from_ptr).ok_or(())
    }
}

impl Drop for CompilationDatabase {
    fn drop(&mut self) {
        unsafe {
            clang_sys::clang_CompilationDatabase_dispose(self.ptr);
        }
    }
}

/// The result of a search in a CompilationDatabase
#[derive(Debug)]
pub struct CompileCommands {
    ptr: CXCompileCommands,
}

impl CompileCommands {
    fn from_ptr(ptr: CXCompileCommands) -> CompileCommands {
        assert!(!ptr.is_null());
        CompileCommands { ptr }
    }

    /// Returns all commands for this search
    pub fn get_commands(&self) -> Vec<CompileCommand> {
        let count = unsafe { clang_CompileCommands_getSize(self.ptr) };
        (0..count)
            .map(|i| unsafe { clang_CompileCommands_getCommand(self.ptr, i) })
            .filter_map(|p| p.map(|p| CompileCommand::from_ptr(self, p)))
            .collect()
    }
}

impl Drop for CompileCommands {
    fn drop(&mut self) {
        unsafe {
            clang_CompileCommands_dispose(self.ptr);
        }
    }
}

pub struct CompileCommand {
    ptr: CXCompileCommand,
}

impl CompileCommand {
    fn from_ptr(_: &CompileCommands, ptr: CXCompileCommand) -> CompileCommand {
        assert!(!ptr.is_null());
        CompileCommand { ptr }
    }

    /// Get the command line arguments for this compile command.
    pub fn get_arguments(&self) -> Vec<String> {
        let count = unsafe { clang_CompileCommand_getNumArgs(self.ptr) };
        (0..count)
            .map(|i| utility::to_string(unsafe { clang_CompileCommand_getArg(self.ptr, i) }))
            .collect()
    }

    /// Get the working directory where the command was executed.
    pub fn get_directory(&self) -> PathBuf {
        utility::to_path(unsafe { clang_CompileCommand_getDirectory(self.ptr) })
    }

    /// Get the filename associated with the command.
    pub fn get_filename(&self) -> PathBuf {
        utility::to_path(unsafe { clang_CompileCommand_getFilename(self.ptr) })
    }

    pub fn get_mapped_source(&self) -> Vec<(String, String)> {
        let count = unsafe { clang_CompileCommand_getNumMappedSources(self.ptr) };
        (0..count)
            .map(|i| {
                let source_path = utility::to_string(unsafe {
                    clang_CompileCommand_getMappedSourcePath(self.ptr, i)
                });
                let source_content = utility::to_string(unsafe {
                    clang_CompileCommand_getMappedSourceContent(self.ptr, i)
                });
                (source_path, source_content)
            })
            .collect()
    }
}
