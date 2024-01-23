#include <sys/param.h>
#include <sys/ioctl.h>

#define PRIVATE 1
#include "pktap.h"
#include "bpf.h"

// included header is missing "want_pktap"
//#include <pcap/pcap.h>
#include "pcap.h"

/* workaround for lack of macro expansions in bindgen */
const uint64_t _BIOCSWANTPKTAP = BIOCSWANTPKTAP;
#undef BIOCSWANTPKTAP
const uint64_t BIOCSWANTPKTAP = _BIOCSWANTPKTAP;
