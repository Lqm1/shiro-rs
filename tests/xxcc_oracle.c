/* Compile with original_xxcc.c staged from pinned SHIRO, changing only its
   ciglet include path. The main rename exposes the unchanged static pipeline. */
#define main original_cli_main
#include "original_xxcc.c"
#undef main
#include <stdint.h>
static void integer(uint32_t value) {if(fwrite(&value,4,1,stdout)!=1) exit(3);}
static void scalar(float value) {if(fwrite(&value,4,1,stdout)!=1) exit(3);}
int main(int argc,char **argv) {
    if(argc!=3) return 2;
    if(!freopen(argv[1],"wb",stdout)) return 3;
    if(fwrite("XCC1",1,4,stdout)!=4) return 3;
    integer(72);
    char *types[]={"mfcc","mfbe","plpcc"};
    int record=0;
    for(int kind=0;kind<3;kind++)
    for(int mode=0;mode<3;mode++)
    for(int flags=0;flags<8;flags++,record++) {
        opt_featuretype=types[kind];opt_order=12;opt_nchannel=13;
        opt_framesize=flags&1?512:511;opt_hopsize=flags&2?80.5:80;
        opt_fs=16000;opt_minbw=400;opt_warp=0.85;
        opt_0=flags&1?1:0;opt_d=flags&2?1:0;opt_a=flags&4?1:0;
        opt_e=mode!=0;opt_E=mode==2?1:0;input_raw=argv[2];
        float signal[321];
        FILE *input=fopen(argv[2],"wb");if(!input) return 3;
        for(int sample=0;sample<321;sample++) {
            if(record%3==0) signal[sample]=0;
            else if(record%3==1) signal[sample]=sample==120?0.75:0;
            else signal[sample]=(sample%17-8)*0.0625;
        }
        if(fwrite(signal,4,321,input)!=321) return 3;
        if(fclose(input)!=0) return 3;
        integer(kind);integer(mode);integer(flags);integer(opt_framesize);
        scalar(opt_hopsize);integer(321);
        for(int sample=0;sample<321;sample++) scalar(signal[sample]);
        integer((int)(321/opt_hopsize));
        integer((12+opt_0+opt_e)*(1+opt_d+opt_a));
        main_xxcc();
    }
    return fclose(stdout)!=0?3:0;
}
