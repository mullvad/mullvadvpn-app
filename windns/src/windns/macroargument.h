#pragma once

//http://www.neff.co.at/2017/04/04/Overloading-Macros-on-the-Number-of-Arguments.html

// Some auxiliary macros
#define EMPTY()
#define EXPAND(X) X
#define CONCAT(X,Y) X##Y

// Get number of arguments passed by __VA_ARGS__
// http://stackoverflow.com/questions/2124339/c-preprocessor-va-args-number-of-arguments
// NUMARGS: all arguments must be castable to int
// NARGS: pure preprocessor macro, maximum 9 arguments
#define NUMARGS(...)  (sizeof((int[]){__VA_ARGS__})/sizeof(int))
#define NARGS(...)  _NARGS_I(_AUGMENT(__VA_ARGS__))
#define _AUGMENT(...) _UNUSED_, __VA_ARGS__
#define _NARGS_I(...) EXPAND(_ARG_N(__VA_ARGS__, 8, 7, 6, 5, 4, 3, 2, 1, 0))
#define _ARG_N(_1, _2, _3, _4, _5, _6, _7, _8, _9, count, ...) count

// Overloading Macro on Number of Arguments
// http://stackoverflow.com/questions/11761703/overloading-macro-on-number-of-arguments
#define _VFUNC(NAME, N) CONCAT(NAME, N)
#define VFUNC(FUNC, ...) CONCAT(_VFUNC(FUNC, NARGS(__VA_ARGS__))(__VA_ARGS__),EMPTY())
