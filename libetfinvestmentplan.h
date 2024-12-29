#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct CEtfInfo {
  const char *id;
  const char *name;
  const char *isin;
} CEtfInfo;

typedef struct CInvestment {
  const char *etf_id;
  const char *name;
  int64_t quantity;
} CInvestment;

typedef struct CInvestments {
  const struct CInvestment *investments;
  uintptr_t length;
} CInvestments;

typedef struct CEtfSetting {
  const char *id;
  const char *isin;
  const char *name;
  double ideal_proportion;
  int64_t cumulative;
} CEtfSetting;

typedef struct CSettings {
  int64_t budget;
  const struct CEtfSetting *etf_settings;
  uintptr_t num_etf_settings;
} CSettings;

const struct CEtfInfo *search_etf_info(const char *etf_isin_ptr);

double get_price_of(const char *etf_id_ptr);

struct CInvestments suggest_investments(void);

int64_t persist_settings(const struct CSettings *settings);

const struct CSettings *get_settings(void);
