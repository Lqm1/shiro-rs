#include <stdint.h>
#include "ciglet.h"
int main(int argc,char **argv) {
    if(argc!=2) return 2;
    FILE *file=fopen(argv[1],"wb");if(!file) return 3;
    uint32_t header[]={64,RAND_MAX};
    if(fwrite("DTH1",1,4,file)!=4 || fwrite(header,4,2,file)!=2) return 3;
    srand(1);
    for(int i=0;i<64;i++) {
        uint32_t value=rand();float uniform=(float)value/RAND_MAX;
        if(fwrite(&value,4,1,file)!=1 || fwrite(&uniform,4,1,file)!=1) return 3;
    }
    srand(1);
    for(int i=0;i<64;i++) {
        float value=randu();if(fwrite(&value,4,1,file)!=1) return 3;
    }
    return fclose(file)!=0?3:0;
}
