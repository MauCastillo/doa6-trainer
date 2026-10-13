/*
 * DOA6.exe falso para probar el trainer bajo Wine sin el juego.
 *
 * Contiene las mismas secuencias de instrucciones que buscan las firmas del
 * catálogo (src/domain/doa6.rs) y las ejecuta en bucle sobre datos propios, de
 * modo que las code caves que inyecta el trainer se ejecutan de verdad. Cada
 * tick publica los resultados en `observed`, que el verificador lee de memoria.
 *
 * Compilar: x86_64-w64-mingw32-gcc -O1 -o DOA6.exe fake_doa6.c
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <windows.h>

void player_tick(void *fighter, int *aux);
void apply_damage(void *fighter, int damage);
void gauge_set(void *gauge, int value);
unsigned speed_read(void *clock);
int challenge_check(void *progress, int target);
void timer_tick(void *timer, float delta);
float rate_calc(void *object, long long divisor);
void score_add(void *score, long long points);
void total_add(void *total, long long value);
extern unsigned char money_check[], site_costumes[], site_hair[], site_hair_menu[], site_glasses[],
    site_music[], site_encyclopedia[], site_trivia[];

void *money_ptr;

/* Las firmas solo dependen de los bytes; todo va escrito con .byte para que el
 * ensamblador no elija otra codificación equivalente. */
__asm__(
    ".intel_syntax noprefix\n"
    ".text\n"
    ".p2align 4\n"

    /* Num 1 / hook del jugador: cmp word [rax+0x100], 0x14 */
    ".globl player_tick\n"
    "player_tick:\n"
    "  push rbx\n"
    "  mov rax, rcx\n"
    "  mov rbx, rdx\n"
    "  .byte 0x66,0x83,0xB8,0x00,0x01,0x00,0x00,0x14\n" /* cmp word [rax+0x100],0x14 */
    "  .byte 0x77,0x0E\n"                               /* ja fin */
    "  .byte 0x8B,0x03\n"                               /* mov eax,[rbx] */
    "  .byte 0x3D,0x11,0x11,0x00,0x00\n"                /* cmp eax,0x1111 */
    "  .byte 0x74,0x05\n"                               /* je fin */
    "  .byte 0x3D,0x22,0x22,0x00,0x00\n"                /* cmp eax,0x2222 */
    "  pop rbx\n"
    "  ret\n"
    ".p2align 4, 0xCC\n"

    /* Num 0 / K.O. de un golpe: vida = max(0, vida - daño) con tope 1000 */
    ".globl apply_damage\n"
    "apply_damage:\n"
    "  push rsi\n"
    "  mov rsi, rcx\n"
    "  movzx ecx, word ptr [rsi+0x100]\n"
    "  mov eax, 1000\n"
    "  .byte 0x66,0x2B,0xCA\n"                          /* sub cx,dx */
    "  .byte 0x66,0x3B,0xC1\n"                          /* cmp ax,cx */
    "  .byte 0x66,0x0F,0x42,0xC8\n"                     /* cmovb cx,ax */
    "  .byte 0x66,0x89,0x8E,0x00,0x01,0x00,0x00\n"      /* mov [rsi+0x100],cx */
    "  pop rsi\n"
    "  ret\n"
    ".p2align 4, 0xCC\n"

    /* Num 2 / Num 9 / barra de Break: [rdi+8]=dueño, [rdi+0x1C]=máximo, [rdi+0x20]=actual */
    ".globl gauge_set\n"
    "gauge_set:\n"
    "  push rdi\n"
    "  mov rdi, rcx\n"
    "  mov ecx, edx\n"
    "  .byte 0x83,0x7F,0x18,0x02\n"                     /* cmp dword [rdi+0x18],2 */
    "  .byte 0x89,0x4F,0x20\n"                          /* mov [rdi+0x20],ecx */
    "  .byte 0x75,0x04\n"                               /* jne fin */
    "  .byte 0x85,0xC9\n"                               /* test ecx,ecx */
    "  .byte 0x7E,0x00\n"                               /* jle fin */
    "  pop rdi\n"
    "  ret\n"
    ".p2align 4, 0xCC\n"

    /* Num 7 / Num 8 / velocidad: devuelve el float de [r14+0x100] */
    ".globl speed_read\n"
    "speed_read:\n"
    "  push r14\n"
    "  sub rsp, 0x20\n"
    "  mov r14, rcx\n"
    "  .byte 0x41,0x8B,0x8E,0x00,0x01,0x00,0x00\n"      /* mov ecx,[r14+0x100] */
    "  .byte 0x41,0x89,0xC8\n"                          /* mov r8d,ecx */
    "  .byte 0xC7,0x44,0x24,0x08,0x01,0x00,0x00,0x00\n" /* mov dword [rsp+8],1 */
    "  .byte 0x33,0xD2\n"                               /* xor edx,edx */
    "  mov eax, ecx\n"
    "  add rsp, 0x20\n"
    "  pop r14\n"
    "  ret\n"
    ".p2align 4, 0xCC\n"

    /* Num 4 / desafíos: devuelve progreso >= objetivo */
    ".globl challenge_check\n"
    "challenge_check:\n"
    "  push rsi\n"
    "  mov rsi, rcx\n"
    "  mov eax, edx\n"
    "  .byte 0x39,0x86,0x00,0x01,0x00,0x00\n"           /* cmp [rsi+0x100],eax */
    "  .byte 0x0F,0x9D,0xC0\n"                          /* setge al */
    "  .byte 0xEB,0x07\n"                               /* jmp fin */
    "  .byte 0x32,0xC0\n"                               /* xor al,al */
    "  .byte 0x4C,0x8B,0x64,0x24,0x08\n"                /* mov r12,[rsp+8] */
    "  movzx eax, al\n"
    "  pop rsi\n"
    "  ret\n"
    ".p2align 4, 0xCC\n"

    /* Num 5 / tiempo: [rsi+8] += delta */
    ".globl timer_tick\n"
    "timer_tick:\n"
    "  push rsi\n"
    "  mov rsi, rcx\n"
    "  movaps xmm0, xmm1\n"
    "  .byte 0xF3,0x0F,0x58,0x46,0x08\n"                /* addss xmm0,[rsi+8] */
    "  .byte 0xF3,0x0F,0x11,0x46,0x08\n"                /* movss [rsi+8],xmm0 */
    "  .byte 0x0F,0xB6,0x86,0x00,0x01,0x00,0x00\n"      /* movzx eax,byte [rsi+0x100] */
    "  .byte 0x66,0x0F,0x6E,0xC0\n"                     /* movd xmm0,eax */
    "  pop rsi\n"
    "  ret\n"
    ".p2align 4, 0xCC\n"

    /* Num 5 (segundo parche): devuelve [rcx+0x14] / divisor + [rbx+0x100] */
    ".globl rate_calc\n"
    "rate_calc:\n"
    "  push rbx\n"
    "  mov rbx, rcx\n"
    "  mov rax, rdx\n"
    "  .byte 0xF3,0x0F,0x10,0x49,0x14\n"                /* movss xmm1,[rcx+0x14] */
    "  .byte 0xF3,0x48,0x0F,0x2A,0xC0\n"                /* cvtsi2ss xmm0,rax */
    "  .byte 0xF3,0x0F,0x5E,0xC8\n"                     /* divss xmm1,xmm0 */
    "  .byte 0x0F,0x57,0xC0\n"                          /* xorps xmm0,xmm0 */
    "  .byte 0xF3,0x0F,0x58,0x8B,0x00,0x01,0x00,0x00\n" /* addss xmm1,[rbx+0x100] */
    "  movaps xmm0, xmm1\n"
    "  pop rbx\n"
    "  ret\n"
    ".p2align 4, 0xCC\n"

    /* Num 6 / puntuación: [rbx+0x130] += puntos * [rbx+0x100] */
    ".globl score_add\n"
    "score_add:\n"
    "  push rbx\n"
    "  mov rbx, rcx\n"
    "  mov rax, rdx\n"
    "  .byte 0x48,0x0F,0xAF,0x83,0x00,0x01,0x00,0x00\n"                     /* imul rax,[rbx+0x100] */
    "  .byte 0x48,0xC7,0x83,0x20,0x01,0x00,0x00,0x00,0x00,0x00,0x00\n"      /* mov qword [rbx+0x120],0 */
    "  .byte 0x48,0x01,0x83,0x30,0x01,0x00,0x00\n"                          /* add [rbx+0x130],rax */
    "  pop rbx\n"
    "  ret\n"
    ".p2align 4, 0xCC\n"

    /* Num 6 (segundo parche): [rcx+0x200] += valor */
    ".globl total_add\n"
    "total_add:\n"
    "  mov rax, rdx\n"
    "  .byte 0x48,0x01,0x81,0x00,0x02,0x00,0x00\n"                          /* add [rcx+0x200],rax */
    "  .byte 0x48,0xC7,0x81,0x10,0x02,0x00,0x00,0x00,0x00,0x00,0x00\n"      /* mov qword [rcx+0x210],0 */
    "  .byte 0x49,0x8B,0xC0\n"                                              /* mov rax,r8 */
    "  .byte 0x8B,0x41,0x10\n"                                              /* mov eax,[rcx+0x10] */
    "  ret\n"
    ".p2align 4, 0xCC\n"

    /* Ctrl+Num 1 / dinero: solo se escanea, nunca se ejecuta */
    ".globl money_check\n"
    "money_check:\n"
    "  mov rdx, qword ptr money_ptr[rip]\n"
    "  .byte 0xB8,0x34,0x12,0x00,0x00\n"                /* mov eax,0x1234 */
    "  .byte 0x41,0x3B,0xC0\n"                          /* cmp eax,r8d */
    "  .byte 0x48,0x0F,0x42,0xC1\n"                     /* cmovb rax,rcx */
    "  .byte 0xF6,0x84,0x02,0x00,0x00,0x00,0x00,0x02\n" /* test byte [rdx+rax],2 */
    "  ret\n"
    ".p2align 4, 0xCC\n"

    /* Parches de sustitución: solo se comprueba que el trainer escribe y restaura */
    ".globl site_costumes\n"
    "site_costumes: .byte 0x48,0x03,0xC3,0xF6,0x00,0x1A,0x0F,0x95,0xC0,0x48,0x83,0xC4,0x20,0x5B,0xC3,"
    "0x48,0x05,0x20,0x4E,0x00,0x00,0xF6,0x00,0x1A\n"
    ".p2align 4, 0xCC\n"
    ".globl site_hair\n"
    "site_hair: .byte 0xEB,0x06,0x48,0x05,0xE8,0x03,0x00,0x00,0xF6,0x00,0x0A\n"
    ".p2align 4, 0xCC\n"
    ".globl site_hair_menu\n"
    "site_hair_menu: .byte 0xB8,0xE8,0x03,0x00,0x00,0x11,0x22,0x33,0x44,0x55,0x66,0x77,0xF6,0x04,0x01,0x0A,"
    "0x0F,0x84,0x10,0x20,0x00,0x00\n"
    ".p2align 4, 0xCC\n"
    ".globl site_glasses\n"
    "site_glasses: .byte 0x48,0x05,0x40,0x1F,0x00,0x00,0xF6,0x00,0x0A,0x75,0x01,0x40,0x84,0xF6,0x75,0x02,"
    "0x31,0xC0\n"
    ".p2align 4, 0xCC\n"
    ".globl site_music\n"
    "site_music: .byte 0x0F,0xB6,0x94,0x08,0x66,0x13,0x00,0x00,0x48,0x8B,0x4C,0x24,0x30,0x80,0xE2,0x01,"
    "0x88,0x54,0x24,0x20,0xE8\n"
    ".p2align 4, 0xCC\n"
    ".globl site_encyclopedia\n"
    "site_encyclopedia: .byte 0x0F,0xB6,0x94,0x08,0xC0,0x15,0x00,0x00,0x48,0x8B,0x4C,0x24,0x30,0x80,0xE2,0x01,"
    "0x88,0x54,0x24,0x20,0xE8\n"
    ".p2align 4, 0xCC\n"
    ".globl site_trivia\n"
    "site_trivia: .byte 0x0F,0xB6,0x94,0x08,0x93,0x14,0x00,0x00,0x48,0x8B,0x4C,0x24,0x30,0x80,0xE2,0x01,"
    "0x88,0x54,0x24,0x20,0xE8\n"
    ".p2align 4, 0xCC\n"
    ".att_syntax\n");

/* Resultados de cada tick; el verificador conoce este orden (todo int32). */
typedef struct {
    volatile int ticks;
    volatile int player_hp;          /* tras player_tick con vida 100 */
    volatile int enemy_hp_after_hit; /* vida 500, golpe de 10 */
    volatile int player_hp_after_hit;
    volatile int player_gauge;       /* actual 10, nuevo valor 20, máximo 1000 */
    volatile int enemy_gauge;
    volatile int speed_bits;         /* float 1.0 sin trucos */
    volatile int challenge_ok;       /* progreso 3 de 10 */
    volatile int challenge_progress;
    volatile int timer_bits;         /* 10.0 - 0.5 */
    volatile int rate_bits;          /* 1.0 / 1 */
    volatile int rate_field_bits;
    volatile int score_total;        /* 5 puntos * 2 */
    volatile int score_slot;
    volatile int total;              /* 5 + 3 */
    volatile int money;              /* 500 hasta que se aplique el truco */
} Observed;

static Observed observed;

typedef struct {
    __attribute__((aligned(16))) unsigned char bytes[0x300];
} Blob;

static Blob player, enemy, player_gauge, enemy_gauge, game_clock, progress, timer, rate, score, total;

#define FIELD(blob, type, offset) (*(type *)((blob).bytes + (offset)))

static void reset_gauge(Blob *gauge, Blob *owner) {
    FIELD(*gauge, void *, 0x08) = owner;
    FIELD(*gauge, int, 0x18) = 2;
    FIELD(*gauge, int, 0x1C) = 1000;
    FIELD(*gauge, int, 0x20) = 10;
}

static int float_bits(float value) {
    int bits;
    memcpy(&bits, &value, sizeof bits);
    return bits;
}

static void tick(void) {
    int aux = 0;
    FIELD(player, short, 0x100) = 100;
    player_tick(&player, &aux);
    observed.player_hp = FIELD(player, short, 0x100);

    FIELD(enemy, short, 0x100) = 500;
    apply_damage(&enemy, 10);
    observed.enemy_hp_after_hit = FIELD(enemy, short, 0x100);
    FIELD(player, short, 0x100) = 500;
    apply_damage(&player, 10);
    observed.player_hp_after_hit = FIELD(player, short, 0x100);

    reset_gauge(&player_gauge, &player);
    gauge_set(&player_gauge, 20);
    observed.player_gauge = FIELD(player_gauge, int, 0x20);
    reset_gauge(&enemy_gauge, &enemy);
    gauge_set(&enemy_gauge, 20);
    observed.enemy_gauge = FIELD(enemy_gauge, int, 0x20);

    FIELD(game_clock, float, 0x100) = 1.0f;
    observed.speed_bits = (int)speed_read(&game_clock);

    FIELD(progress, int, 0x100) = 3;
    observed.challenge_ok = challenge_check(&progress, 10);
    observed.challenge_progress = FIELD(progress, int, 0x100);

    FIELD(timer, float, 0x08) = 10.0f;
    timer_tick(&timer, -0.5f);
    observed.timer_bits = FIELD(timer, int, 0x08);

    FIELD(rate, float, 0x14) = 1.0f;
    FIELD(rate, float, 0x100) = 0.0f;
    observed.rate_bits = float_bits(rate_calc(&rate, 1));
    observed.rate_field_bits = FIELD(rate, int, 0x14);

    FIELD(score, long long, 0x100) = 2;
    FIELD(score, int, 0x108) = 0;
    FIELD(score, long long, 0x130) = 0;
    score_add(&score, 5);
    observed.score_total = (int)FIELD(score, long long, 0x130);
    observed.score_slot = FIELD(score, int, 0x108);

    FIELD(total, long long, 0x200) = 5;
    total_add(&total, 3);
    observed.total = (int)FIELD(total, long long, 0x200);

    observed.money = *(int *)money_ptr;
    observed.ticks++;
}

int main(void) {
    money_ptr = calloc(1, 64);
    *(int *)money_ptr = 500;

    FILE *state = fopen("fake_doa6_state.txt", "w");
    if (!state) {
        return 1;
    }
    fprintf(state, "observed=%p\n", (void *)&observed);
    fprintf(state, "money=%p\n", money_ptr);
    fprintf(state, "costumes=%p\n", (void *)site_costumes);
    fprintf(state, "hair=%p\n", (void *)site_hair);
    fprintf(state, "hair_menu=%p\n", (void *)site_hair_menu);
    fprintf(state, "glasses=%p\n", (void *)site_glasses);
    fprintf(state, "music=%p\n", (void *)site_music);
    fprintf(state, "encyclopedia=%p\n", (void *)site_encyclopedia);
    fprintf(state, "trivia=%p\n", (void *)site_trivia);
    fclose(state);

    for (;;) {
        tick();
        Sleep(5);
    }
}
