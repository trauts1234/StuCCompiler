int main()
{
	int x[2] = {1,2};
	if(x[0] != 1){return 1;}
	if(x[1] != 2){return 2;}

	char y[2] = {1u, 2ull + 1ull};
	if(y[0] != 1){return 3;}
	if(y[1] != 3){return 4;}

	int a = 1;
	int b = 2;
	int z[2] = {a, a+b};
	if(z[0] != 1){return 5;}
	if(z[1] != 3){return 6;}
}