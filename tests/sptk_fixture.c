/* Protocol fixture only: this does not implement SPTK feature mathematics. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif
int main(int argc,char **argv) {
#ifdef _WIN32
    _setmode(_fileno(stdin),_O_BINARY);_setmode(_fileno(stdout),_O_BINARY);
#endif
    char *name=argv[0];
    for(char *p=name;*p;p++) if(*p=='/' || *p=='\\') name=p+1;
    int role=strncmp(name,"frame",5)==0 ? 0 : strncmp(name,"mfcc",4)==0 ? 1 : 2;
    if(strstr(name,"-fail")) return 7;
    const char *expected_frame[]={"-l","512","-p","80"};
    const char *expected_mfcc[]={"-l","512","-m","12","-s","16"};
    const char *expected_delta[]={"-l","12","-d","-0.5","0","0.5","-d","0.25","0","-0.5","0","0.25"};
    const char **expected=role==0 ? expected_frame : role==1 ? expected_mfcc : expected_delta;
    int count=role==0 ? 4 : role==1 ? 6 : 12;
    if(argc!=count+1+(role!=1)) return 4;
    for(int i=0;i<count;i++) if(strcmp(argv[i+1],expected[i])) return 5;
    FILE *input=role==1 ? stdin : fopen(argv[argc-1],"rb");if(!input) return 3;
    unsigned char buffer[16384];size_t count_read;
    while((count_read=fread(buffer,1,sizeof(buffer),input))>0) if(fwrite(buffer,1,count_read,stdout)!=count_read) return 3;
    if(ferror(input)) return 3;
    if(input!=stdin && fclose(input)!=0) return 3;
    return fflush(stdout)!=0 ? 3 : 0;
}
