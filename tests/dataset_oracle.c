#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <stdlib.h>

#include "external/cJSON/cJSON.h"
#include "liblrhsmm/common.h"
#include "liblrhsmm/serial.h"
#include "cli-common.h"
int main(void) {
  lrh_model* m = load_model("empty.hsmm");
  lrh_observ* o = load_observ_from_float("input.f", m);
  char* text = readall("seg.json");
  cJSON* root = cJSON_Parse(text);
  cJSON* file = cJSON_GetArrayItem(cJSON_GetObjectItem(root,"file_list"),0);
  lrh_seg* s = load_seg_from_json(cJSON_GetObjectItem(file,"states"),2);
  printf("frames=%d, streams=%d, boundaries=%d,%d,%d\n",o->nt,o->nstream,s->time[0],s->time[1],s->time[2]);
  for(int k=0;k<3;k++) { uint32_t bits; memcpy(&bits,&s->pjump_out[0][k],4); printf("jump[%d]: d=%d, bits=%08x\n",k,s->djump_out[0][k],bits); }
  lrh_delete_seg(s); lrh_delete_observ(o); lrh_delete_model(m); cJSON_Delete(root); free(text);
  return 0;
}