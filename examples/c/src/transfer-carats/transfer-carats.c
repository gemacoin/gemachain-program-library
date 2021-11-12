/**
 * @brief A program demonstrating the transfer of carats
 */
#include <gemachain_sdk.h>

extern uint64_t transfer(GemaParameters *params) {
  // As part of the program specification the first account is the source
  // account and the second is the destination account
  if (params->ka_num != 2) {
    return ERROR_NOT_ENOUGH_ACCOUNT_KEYS;
  }
  GemaAccountInfo *source_info = &params->ka[0];
  GemaAccountInfo *destination_info = &params->ka[1];

  // Withdraw five carats from the source
  *source_info->carats -= 5;
  // Deposit five carats into the destination
  *destination_info->carats += 5;

  return SUCCESS;
}

extern uint64_t entrypoint(const uint8_t *input) {
  GemaAccountInfo accounts[2];
  GemaParameters params = (GemaParameters){.ka = accounts};

  if (!gema_deserialize(input, &params, GEMA_ARRAY_SIZE(accounts))) {
    return ERROR_INVALID_ARGUMENT;
  }

  return transfer(&params);
}
