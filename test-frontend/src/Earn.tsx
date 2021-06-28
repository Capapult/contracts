import React from 'react';
import {
  Button,
  DialogTitle,
  DialogContent,
  DialogActions
} from "@material-ui/core";
import { useDialog } from "./DialogProvider";

function Earn() {
  let totaldeposit = 10000;
  
  const [openDialog, closeDialog] = useDialog();
  const onDepositClick = () => {
    openDialog({
      children: (
        <>
          <DialogTitle>Deposit</DialogTitle>
          <DialogContent>
            Lorem ipsum dolor sit amet consectetur, adipisicing elit. Nobis
            totam accusamus corporis, aliquid optio accusantium expedita nihil
            illo qui, commodi voluptatibus? Ducimus nesciunt animi, nulla rem at
            obcaecati aperiam eos!
          </DialogContent>
          <DialogActions>
            <Button color="primary" onClick={() => closeDialog()}>
              Close
            </Button>
          </DialogActions>
        </>
      )
    });
  };
  const onWithdrawClick = () => {
    openDialog({
      children: (
        <>
          <DialogTitle>Withdraw</DialogTitle>
          <DialogContent>Nothing much here</DialogContent>
          <DialogActions>
            <Button color="primary" onClick={() => closeDialog()}>
              Close
            </Button>
          </DialogActions>
        </>
      )
    });
  };

  return (
    <div className="Earn">
      <h3>Total deposit: {totaldeposit}</h3> 

      <Button variant="outlined" color="primary" onClick={onDepositClick}> Deposit </Button>&nbsp;&nbsp;&nbsp;  
      <Button variant="outlined" color="primary"  onClick={onWithdrawClick}> Withdraw </Button>   
    </div>
  );
}

export default Earn;
