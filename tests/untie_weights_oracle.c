#include <stdio.h>
#include <stdlib.h>
#include "external/cJSON/cJSON.h"
#include "liblrhsmm/common.h"
#include "liblrhsmm/serial.h"
#include "cli-common.h"

int main(int argc, char** argv) {
  if (argc != 2) return 1;
  lrh_model* model = load_model(argv[1]);
  if (!model || model->nstream != 2) return 1;
  model->streams[0]->weight = 0.25f;
  model->streams[1]->weight = 1.75f;
  cmp_ctx_t context;
  cmp_init(&context, stdout, file_reader, file_writer);
  lrh_write_model(&context, model);
  lrh_delete_model(model);
  return 0;
}
