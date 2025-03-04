int main() {
    {
        int i=0;//uses some stack space
    }
    {
        int j=1;//this may use the same stack space as "i" used to, but i is now out of scope
        return j;
    }
}