// #include "logging.c"
// #include <criterion/criterion.h>

// Test(logging, sanity) {
//   uint8_t instruction_data[] = {10, 11, 12, 13, 14};
//   GemaPubkey program_id = {.x = {
//                               1,
//                           }};
//   GemaPubkey key = {.x = {
//                        2,
//                    }};
//   uint64_t carats = 1;
//   uint8_t data[] = {0, 0, 0, 0};
//   GemaAccountInfo accounts[] = {};
//   GemaParameters params = {accounts, sizeof(accounts) /
//   sizeof(GemaAccountInfo), instruction_data,
//                           sizeof(instruction_data), &program_id};

//   cr_assert(SUCCESS == logging(&params));
// }
